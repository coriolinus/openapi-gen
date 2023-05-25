use heck::AsUpperCamelCase;
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::codegen::make_ident;

use super::{default_derives, OneOfEnum, Scalar, Value};

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
pub struct Item {
    /// Documentation to be injected for this item.
    pub docs: Option<String>,
    /// Name defined inline. Overrides the generated name.
    pub name: Option<String>,
    /// When true, construct a newtype instead of a typedef.
    pub newtype: bool,
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
    pub value: Value,
}

impl From<Value> for Item {
    fn from(value: Value) -> Self {
        Item {
            value,
            ..Default::default()
        }
    }
}

/// Code generation is going to be a whole lot easier given an AST that maps to Rust types, rather than one which
/// conforms to OpenAPI semantics. So, let's build such an AST.
impl<'a> TryFrom<&'a Schema> for Item {
    type Error = String;

    fn try_from(schema: &'a Schema) -> Result<Self, Self::Error> {
        let value = match &schema.schema_kind {
            SchemaKind::Any(_) => Scalar::Any.into(),
            SchemaKind::Type(Type::Boolean {}) => Value::Scalar(Scalar::Bool),
            SchemaKind::Type(Type::String(string_type)) => {
                Value::try_from_string_type(string_type, &schema.schema_data)?
            }
            SchemaKind::Type(Type::Number(number_type)) => number_type.try_into()?,
            SchemaKind::Type(Type::Integer(integer_type)) => integer_type.try_into()?,
            SchemaKind::Type(Type::Array(array_type)) => array_type.try_into()?,
            SchemaKind::Type(Type::Object(object_type)) => object_type.try_into()?,
            SchemaKind::OneOf { one_of } => {
                OneOfEnum::try_from(&schema.schema_data, one_of)?.into()
            }
            SchemaKind::AllOf { .. } | SchemaKind::AnyOf { .. } | SchemaKind::Not { .. } => {
                return Err("`allOf`, `anyOf`, and `not` are not supported".into())
            }
        };

        // Get documentation from the provided external documentation, or alternately from the description.
        let docs = schema
            .schema_data
            .external_docs
            .as_ref()
            .map(|docs| reqwest::blocking::get(&docs.url))
            .transpose()
            .map_err(|err| err.to_string())?
            .map(|response| response.text())
            .transpose()
            .map_err(|err| err.to_string())?
            .or_else(|| schema.schema_data.description.clone());

        let name = schema.schema_data.title.clone().or_else(|| {
            get_extension_value(schema, "name")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
        });

        let newtype = get_extension_bool(schema, "newtype");
        let newtype_from = get_extension_bool(schema, "newtypeFrom");
        let newtype_into = get_extension_bool(schema, "newtypeInto");
        let newtype_deref = get_extension_bool(schema, "newtypeDeref");
        let newtype_deref_mut = get_extension_bool(schema, "newtypeDerefMut");

        let nullable = schema.schema_data.nullable;

        Ok(Self {
            docs,
            name,
            newtype,
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
        !self.newtype && matches!(&self.value, Value::Scalar(_))
    }

    /// Calculate the inner identifier for this item, without filtering by nullability.
    fn inner_ident_unfiltered(&self, derived_name: &str) -> String {
        let name = self.name.as_deref().unwrap_or(derived_name);
        format!("{}", AsUpperCamelCase(name))
    }

    /// Calculate the inner identifier for this item.
    ///
    /// This only exists if the item is nullable.
    pub fn inner_ident(&self, derived_name: &str) -> Option<Ident> {
        self.nullable
            .then(|| make_ident(&self.inner_ident_unfiltered(derived_name)))
    }

    /// Calculate the referent identifier for this item.
    ///
    /// Rules:
    ///
    /// - If there's a defined `name`, we use that instead of the derived name.
    /// - If the item is nullable, we use a `MaybeX` variant.
    pub fn referent_ident(&self, derived_name: &str) -> Ident {
        let inner = self.inner_ident_unfiltered(derived_name);
        let referent = if self.nullable {
            format!("Maybe{inner}")
        } else {
            inner
        };
        make_ident(&referent)
    }

    /// Is this item public?
    ///
    /// True if any:
    ///
    /// - is a struct or enum
    /// - is a newtype
    /// - has a `name`
    pub fn is_pub(&self) -> bool {
        self.newtype || self.value.is_struct_or_enum() || self.name.is_some()
    }

    /// Generate an item definition for this item.
    ///
    /// Uses a depth-first traversal to ensure inline-defined sub-item
    /// definitions appear before containing item definitions. However, makes no
    /// attempt to generate reference items.
    ///
    /// `derived_name` is the derived name for this item, based on its position
    /// in the tree structure.
    pub fn emit(&self, derived_name: &str) -> TokenStream {
        let sub_item_defs = self.value.sub_items().map(|(name_fragment, item)| {
            let derived_name = format!(
                "{}{}",
                AsUpperCamelCase(derived_name),
                AsUpperCamelCase(name_fragment)
            );
            item.emit(&derived_name)
        });

        let docs = self.docs.as_ref().map(|docs| quote!(#[doc = #docs]));

        let wrapper_def = self.inner_ident(derived_name).map(|inner_ident| {
            let outer_ident = self.referent_ident(derived_name);
            quote! {
                type #outer_ident = Option<#inner_ident>;
            }
        });

        let item_keyword = if self.newtype {
            quote!(struct)
        } else {
            self.value.item_keyword()
        };

        let equals = (!self.newtype && !self.value.is_struct_or_enum()).then_some(quote!(=));

        let item_ident = self
            .inner_ident(derived_name)
            .unwrap_or_else(|| self.referent_ident(derived_name));

        // TODO: derives
        let derives = (!self.is_typedef())
            .then(|| self.derives(todo!()))
            .and_then(|derives| (!derives.is_empty()).then_some(derives))
            .map(|derives| {
                quote! {
                    #[derive(
                        #( #derives ),*
                    )]
                }
            });

        let pub_ = self.is_pub().then_some(quote!(pub));

        let item_def = self.value.emit_item_definition(derived_name);

        let semicolon = (self.newtype || !self.value.is_struct_or_enum()).then_some(quote!(;));

        quote! {
            #( #sub_item_defs )*

            #wrapper_def

            #docs
            #derives
            #pub_ #item_keyword #item_ident #equals #item_def #semicolon
        }
    }

    pub fn reference_referent_ident(item_ref: &ReferenceOr<Item>, derived_name: &str) -> Ident {
        match item_ref {
            ReferenceOr::Reference { reference } => {
                let name = reference.rsplit('/').next().unwrap_or(reference.as_ref());
                make_ident(name)
            }
            ReferenceOr::Item(item) => item.referent_ident(derived_name),
        }
    }

    /// The list of derives which should attach to this item.
    ///
    /// TODO: use a registry of some kind instead of the original spec.
    pub fn derives(&self, spec: &OpenAPI) -> Vec<TokenStream> {
        let mut derives = default_derives();

        if self.value.impls_copy(spec) {
            derives.push(quote!(Copy));
        }
        if self.value.impls_eq(spec) {
            derives.push(quote!(Eq));
        }
        if self.value.impls_hash(spec) {
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

        derives
    }
}
