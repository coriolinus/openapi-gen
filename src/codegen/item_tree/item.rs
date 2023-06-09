use heck::ToUpperCamelCase;
use openapiv3::{Schema, SchemaKind, Type};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;

use crate::codegen::make_ident;

use super::{
    api_model::{Ref, Reference, UnknownReference},
    value::ValueConversionError,
    ApiModel, Scalar, Value,
};

// note: openapiv3 intentionally ignores extensions whose name does not start with "x-".
fn get_extension_value<'a>(schema: &'a Schema, key: &str) -> Option<&'a serde_json::Value> {
    schema.schema_data.extensions.get(key)
}

fn get_extension_bool(schema: &Schema, key: &str) -> bool {
    get_extension_value(schema, key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_default()
}

#[derive(Default, Debug, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct NewtypeOptions {
    /// When true, derive [`derive_more::From`].
    from: bool,
    /// When true, derive [`derive_more::Into`].
    into: bool,
    /// When true, derive [`derive_more::Deref`].
    deref: bool,
    /// When true, derive [`derive_more::DerefMut`].
    deref_mut: bool,
    /// When true, the inner item is `pub`.
    #[serde(rename = "pub")]
    pub_: bool,
}

impl NewtypeOptions {
    // orphan rule prevents a normal `From` impl
    fn from(value: serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::Bool(b) => b.then_some(NewtypeOptions::default()),
            serde_json::Value::Object(_) => serde_json::from_value(value).ok(),
            serde_json::Value::Null
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_)
            | serde_json::Value::Array(_) => None,
        }
    }
}

/// Root struct of the abstract item tree used here to model OpenAPI items.
///
/// This ultimately controls everything about how an item is emitted in Rust.
#[derive(Debug, Clone)]
pub struct Item<Ref = Reference> {
    /// Documentation to be injected for this item.
    pub docs: Option<String>,
    /// Name of this item as it appears in the specification.
    pub spec_name: String,
    /// Name of this item as it appears in Rust code.
    pub rust_name: String,
    /// Inner name of this item. Should always and only be `Some` if `self.nullable`.
    pub inner_name: Option<String>,
    /// When `Some`, construct a newtype instead of a typedef.
    pub newtype: Option<NewtypeOptions>,
    /// When true, typedef is public.
    pub pub_typedef: bool,
    /// When true, emits an outer typedef around an `Option`, and an inner item definition.
    pub nullable: bool,
    /// What value this item contains
    pub value: Value<Ref>,
}

impl<R> Default for Item<R> {
    fn default() -> Self {
        Self {
            docs: Default::default(),
            spec_name: Default::default(),
            rust_name: Default::default(),
            inner_name: Default::default(),
            newtype: Default::default(),
            pub_typedef: Default::default(),
            nullable: Default::default(),
            value: Default::default(),
        }
    }
}

impl Item<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Item<Reference>, UnknownReference> {
        let Self {
            docs,
            spec_name,
            rust_name: name,
            inner_name,
            newtype,
            pub_typedef,
            nullable,
            value,
        } = self;
        let value = value.resolve_refs(resolver)?;
        Ok(Item {
            docs,
            spec_name,
            rust_name: name,
            inner_name,
            newtype,
            pub_typedef,
            nullable,
            value,
        })
    }

    /// Parse a schema, recursively adding inline items
    pub(crate) fn parse_schema(
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        schema: &Schema,
    ) -> Result<Self, ParseItemError> {
        let value: Value<Ref> = match &schema.schema_kind {
            SchemaKind::Type(Type::Boolean {}) => Value::Scalar(Scalar::Bool),
            SchemaKind::Type(Type::Number(number_type)) => number_type.try_into()?,
            SchemaKind::Type(Type::Integer(integer_type)) => integer_type.try_into()?,
            SchemaKind::Any(any_schema) => {
                Value::try_parse_string_enum_type(&schema.schema_data, any_schema)
                    .unwrap_or(Scalar::Any.into())
            }
            SchemaKind::Type(Type::String(string_type)) => {
                Value::parse_string_type(string_type, &schema.schema_data)?
            }
            SchemaKind::Type(Type::Array(array_type)) => {
                Value::parse_array_type(model, spec_name, rust_name, array_type)?
            }
            SchemaKind::Type(Type::Object(object_type)) => {
                Value::parse_object_type(model, spec_name, rust_name, object_type)?
            }
            SchemaKind::OneOf { one_of } => {
                Value::parse_one_of_type(model, spec_name, rust_name, &schema.schema_data, one_of)?
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

        let pub_typedef = get_extension_bool(schema, "x-pub-typedef");
        let newtype = get_extension_value(schema, "x-newtype")
            .cloned()
            .and_then(NewtypeOptions::from);

        let nullable = schema.schema_data.nullable;

        // The names used for this item can either be set explicitly with the `title` field, or we can just derive it.
        let (spec_name, rust_name) = schema
            .schema_data
            .title
            .as_ref()
            .map(|title| {
                let mut rust_name = title.to_upper_camel_case();
                model.deconflict_ident(&mut rust_name);
                (title.clone(), rust_name)
            })
            .unwrap_or_else(|| (spec_name.to_owned(), rust_name.to_owned()));

        // the public name and inner name depend on whether this is nullable or not
        let (rust_name, inner_name) = if nullable {
            let mut maybe_name = format!("Maybe{rust_name}");
            model.deconflict_ident(&mut maybe_name);
            (maybe_name, Some(rust_name))
        } else {
            (rust_name, None)
        };

        Ok(Self {
            docs,
            spec_name,
            rust_name,
            inner_name,
            newtype,
            pub_typedef,
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
        self.newtype.is_none()
            && match &self.value {
                Value::Scalar(_)
                | Value::Set(_)
                | Value::List(_)
                | Value::Map(_)
                | Value::Ref(_) => true,
                Value::StringEnum(_) | Value::OneOfEnum(_) | Value::Object(_) => false,
            }
    }

    /// Is this item public?
    ///
    /// True if any:
    ///
    /// - is a struct or enum
    /// - is a newtype
    /// - is marked as public
    pub fn is_pub(&self) -> bool {
        self.newtype.is_some() || self.pub_typedef || self.value.is_struct_or_enum()
    }

    /// Generate an item definition for this item.
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
                let outer_ident = make_ident(&self.rust_name);
                let inner_ident = make_ident(inner);

                (
                    Some(quote! {
                        type #outer_ident = Option<#inner_ident>;
                    }),
                    inner_ident,
                )
            }
            None => (None, make_ident(&self.rust_name)),
        };

        let item_keyword = if self.newtype.is_some() {
            quote!(struct)
        } else {
            self.value.item_keyword()
        };

        let equals =
            (self.newtype.is_none() && !self.value.is_struct_or_enum()).then_some(quote!(=));

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

        let serde_container_attributes = self.value.serde_container_attributes();
        let serde_container_attributes = (!serde_container_attributes.is_empty())
            .then(move || quote!(#[serde( #( #serde_container_attributes ),*)]));

        let pub_ = self.is_pub().then_some(quote!(pub));

        let mut item_def = self.value.emit_item_definition(model, name_resolver)?;
        if self.newtype.map(|options| options.pub_).unwrap_or_default() {
            item_def = quote!(pub #item_def);
        }
        if self.newtype.is_some() {
            item_def = quote!((#item_def));
        }

        let semicolon = (self.newtype.is_some() || self.is_typedef()).then_some(quote!(;));

        Ok(quote! {
            #wrapper_def

            #docs
            #derives
            #serde_container_attributes
            #pub_ #item_keyword #item_ident #equals #item_def #semicolon
        })
    }

    /// The list of derives which should attach to this item.
    pub fn derives(&self, model: &ApiModel) -> Vec<TokenStream> {
        let mut derives = vec![
            quote!(Debug),
            quote!(Clone),
            quote!(PartialEq),
            quote!(openapi_gen::reexport::serde::Serialize),
            quote!(openapi_gen::reexport::serde::Deserialize),
        ];

        if self.value.impls_copy(model) {
            derives.push(quote!(Copy));
        }
        if self.value.impls_eq(model) {
            derives.push(quote!(Eq));
        }
        if self.value.impls_hash(model) {
            derives.push(quote!(Hash));
        }
        if let Some(options) = self.newtype {
            if options.from {
                derives.push(quote!(openapi_gen::reexport::derive_more::From));
            }
            if options.into {
                derives.push(quote!(openapi_gen::reexport::derive_more::Into));
            }
            if options.deref {
                derives.push(quote!(openapi_gen::reexport::derive_more::Deref));
            }
            if options.deref_mut {
                derives.push(quote!(openapi_gen::reexport::derive_more::DerefMut));
            }
        }
        if self.newtype.is_some() || matches!(&self.value, Value::Object(_)) {
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
