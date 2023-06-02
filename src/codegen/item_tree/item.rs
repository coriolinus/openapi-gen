use heck::AsUpperCamelCase;
use openapiv3::{Schema, SchemaKind, Type};
use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::make_ident;

use super::{
    api_model::{Ref, Reference, UnknownReference},
    default_derives,
    value::ValueConversionError,
    ApiModel, Scalar, Value,
};

fn get_extension_value<'a>(schema: &'a Schema, key: &str) -> Option<&'a serde_json::Value> {
    schema.schema_data.extensions.get(key)
}

fn get_extension_bool(schema: &Schema, key: &str) -> bool {
    get_extension_value(schema, key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_default()
}

/// Root struct of the abstract item tree used here to model OpenAPI items.
///
/// This ultimately controls everything about how an item is emitted in Rust.
#[derive(Default, Debug, Clone)]
pub struct Item<Ref = Reference> {
    /// Documentation to be injected for this item.
    pub docs: Option<String>,
    /// A `name` extension field was set, explicitly naming this item.
    pub explicit_name: bool,
    /// Name of this item.
    pub name: String,
    /// Inner name of this item. Should always and only be `Some` if `self.nullable`.
    pub inner_name: Option<String>,
    /// When true, construct a newtype instead of a typedef.
    pub newtype: bool,
    /// When true and `newtype` is set, the newtype inner item is public.
    pub newtype_pub: bool,
    /// When true and `newtype` is set, derive [`derive_more::From`].
    pub newtype_from: bool,
    /// When true and `newtype` is set, derive [`derive_more::Into`].
    pub newtype_into: bool,
    /// When true and `newtype` is set, derive [`derive_more::Deref`].
    pub newtype_deref: bool,
    /// When true and `newtype` is set, derive [`derive_more::DerefMut`].
    pub newtype_deref_mut: bool,
    /// When true, emits an outer typedef around an `Option`, and an inner item definition.
    pub nullable: bool,
    /// What value this item contains
    pub value: Value<Ref>,
}

impl Item<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Item<Reference>, UnknownReference> {
        let Self {
            docs,
            explicit_name,
            name,
            inner_name,
            newtype,
            newtype_pub,
            newtype_from,
            newtype_into,
            newtype_deref,
            newtype_deref_mut,
            nullable,
            value,
        } = self;
        let value = value.resolve_refs(resolver)?;
        Ok(Item {
            docs,
            explicit_name,
            name,
            inner_name,
            newtype,
            newtype_pub,
            newtype_from,
            newtype_into,
            newtype_deref,
            newtype_deref_mut,
            nullable,
            value,
        })
    }

    /// Parse a schema, recursively adding inline items
    pub(crate) fn parse_schema(
        model: &mut ApiModel<Ref>,
        name: &str,
        schema: &Schema,
    ) -> Result<Self, ParseItemError> {
        let value: Value<Ref> = match &schema.schema_kind {
            SchemaKind::Any(_) => Scalar::Any.into(),
            SchemaKind::Type(Type::Boolean {}) => Value::Scalar(Scalar::Bool),
            SchemaKind::Type(Type::Number(number_type)) => number_type.try_into()?,
            SchemaKind::Type(Type::Integer(integer_type)) => integer_type.try_into()?,
            SchemaKind::Type(Type::String(string_type)) => {
                Value::parse_string_type(string_type, &schema.schema_data)?
            }
            SchemaKind::Type(Type::Array(array_type)) => {
                Value::parse_array_type(model, name, array_type)?
            }
            SchemaKind::Type(Type::Object(object_type)) => {
                Value::parse_object_type(model, name, object_type)?
            }
            SchemaKind::OneOf { one_of } => {
                Value::parse_one_of_type(model, name, &schema.schema_data, one_of)?
            }
            SchemaKind::AllOf { .. } | SchemaKind::AnyOf { .. } | SchemaKind::Not { .. } => {
                return Err(ParseItemError::UnsupportedSchemaKind)
            }
        };

        // Get documentation from the provided external documentation link if present, or alternately from the description.
        let docs = schema
            .schema_data
            .external_docs
            .as_ref()
            .map(|docs| reqwest::blocking::get(&docs.url)?.text())
            .transpose()
            .map_err(ParseItemError::ExternalDocumentation)?
            .or_else(|| schema.schema_data.description.clone());

        let newtype = get_extension_bool(schema, "newtype");
        let newtype_pub = get_extension_bool(schema, "newtypePub");
        let newtype_from = get_extension_bool(schema, "newtypeFrom");
        let newtype_into = get_extension_bool(schema, "newtypeInto");
        let newtype_deref = get_extension_bool(schema, "newtypeDeref");
        let newtype_deref_mut = get_extension_bool(schema, "newtypeDerefMut");

        let nullable = schema.schema_data.nullable;

        // The name used for this item can come from one of three sources:
        //
        // - It's the `name` extension value field, or
        // - It's the `title` field (in UpperCamelCase), or
        // - It's the derived name for this item
        let explicit_name = get_extension_value(schema, "name")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);
        let has_explicit_name = explicit_name.is_some();
        let name = {
            let mut name = explicit_name
                .or_else(|| schema.schema_data.title.clone())
                .map(|title| format!("{}", AsUpperCamelCase(title)))
                .unwrap_or_else(|| name.to_owned());
            model.deconflict_ident(&mut name);
            name
        };

        let (name, inner_name) = if nullable {
            let mut maybe_name = format!("Maybe{name}");
            model.deconflict_ident(&mut maybe_name);
            (maybe_name, Some(name))
        } else {
            (name, None)
        };

        debug_assert_eq!(inner_name.is_some(), nullable);

        Ok(Self {
            docs,
            explicit_name: has_explicit_name,
            name,
            inner_name,
            newtype,
            newtype_pub,
            newtype_from,
            newtype_into,
            newtype_deref,
            newtype_deref_mut,
            nullable,
            value,
        })
    }
}

impl Item {
    /// `true` when the item is a typedef.
    ///
    /// This disables derives when the item is emitted.
    fn is_typedef(&self) -> bool {
        !self.newtype
            && match &self.value {
                Value::Scalar(_) | Value::Set(_) | Value::List(_) | Value::Map(_) => true,
                Value::StringEnum(_) | Value::OneOfEnum(_) | Value::Object(_) => false,
            }
    }

    /// Is this item public?
    ///
    /// True if any:
    ///
    /// - is a struct or enum
    /// - is a newtype
    /// - has a `name`
    pub fn is_pub(&self) -> bool {
        self.newtype || self.explicit_name || self.value.is_struct_or_enum()
    }

    /// Generate an item definition for this item.
    ///
    /// `name` is the public name for this item.
    ///
    /// The name resolver should be able to efficiently extract item names from references.
    pub fn emit<'a>(
        &self,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let docs = self.docs.as_ref().map(|docs| quote!(#[doc = #docs]));

        let (wrapper_def, item_ident) = match &self.inner_name {
            Some(inner) => {
                let outer_ident = make_ident(&self.name);
                let inner_ident = make_ident(inner);

                (
                    Some(quote! {
                        type #outer_ident = Option<#inner_ident>;
                    }),
                    inner_ident,
                )
            }
            None => (None, make_ident(&self.name)),
        };

        let item_keyword = if self.newtype {
            quote!(struct)
        } else {
            self.value.item_keyword()
        };

        let equals = (!self.newtype && !self.value.is_struct_or_enum()).then_some(quote!(=));

        let derives = (!self.is_typedef())
            .then(|| self.derives(model))
            .and_then(|derives| (!derives.is_empty()).then_some(derives))
            .map(|derives| {
                quote! {
                    #[derive(
                        #( #derives ),*
                    )]
                }
            });

        let pub_ = self.is_pub().then_some(quote!(pub));

        let mut item_def = self.value.emit_item_definition(model, name_resolver)?;
        if self.newtype {
            let newtype_pub = self.newtype_pub.then_some(quote!(pub));
            item_def = quote!((#newtype_pub #item_def));
        }

        let semicolon = (self.newtype || !self.value.is_struct_or_enum()).then_some(quote!(;));

        Ok(quote! {
            #wrapper_def

            #docs
            #derives
            #pub_ #item_keyword #item_ident #equals #item_def #semicolon
        })
    }

    /// The list of derives which should attach to this item.
    pub fn derives(&self, model: &ApiModel) -> Vec<TokenStream> {
        let mut derives = default_derives();

        if self.value.impls_copy(model) {
            derives.push(quote!(Copy));
        }
        if self.value.impls_eq(model) {
            derives.push(quote!(Eq));
        }
        if self.value.impls_hash(model) {
            derives.push(quote!(Hash));
        }
        if self.newtype {
            if self.newtype_from {
                derives.push(quote!(openapi_gen::reexport::derive_more::From));
            }
            if self.newtype_into {
                derives.push(quote!(openapi_gen::reexport::derive_more::Into));
            }
            if self.newtype_deref {
                derives.push(quote!(openapi_gen::reexport::derive_more::Deref));
            }
            if self.newtype_deref_mut {
                derives.push(quote!(openapi_gen::reexport::derive_more::DerefMut));
            }
        }
        if self.newtype || matches!(&self.value, Value::Object(_)) {
            derives.push(quote!(openapi_gen::reexport::derive_more::Constructor));
        }

        derives
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ParseItemError {
    #[error("could not parse value type")]
    ValueConversion(#[from] ValueConversionError),
    #[error("`allOf`, `anyOf`, and `not` schemas are not supported")]
    UnsupportedSchemaKind,
    #[error("failed to get external documentation")]
    ExternalDocumentation(#[source] reqwest::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("unable to resolve name from reference: {0:?}")]
    UnresolvedReference(Reference),
}
