use std::fmt;

use heck::ToUpperCamelCase;
use openapiv3::{ObjectType, OpenAPI, Schema, SchemaKind, Type};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;

use crate::{
    codegen::{
        make_ident, ApiModel, PropertyOverride, Ref, Reference, Scalar, UnknownReference, Value,
        ValueConversionError,
    },
    resolve_trait::Resolve,
};

use super::{api_model::AsBackref, OneOfEnum, StringEnum};

// note: openapiv3 intentionally ignores extensions whose name does not start with "x-".
fn get_extension_value<'a>(schema: &'a Schema, key: &str) -> Option<&'a serde_json::Value> {
    schema.schema_data.extensions.get(key)
}

fn get_extension_bool(schema: &Schema, key: &str) -> bool {
    get_extension_value(schema, key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_default()
}

/// The object type containing this schema, and the item name
/// of the property identifying this schema.
pub(crate) type ContainingObject<'a> = Option<(&'a ObjectType, &'a str)>;

/// Does this `allOf` instance follow the rules of an `allOf` property singleton?
///
/// The rules for an `allOf` singleton are as follows:
///
/// - the schema is a property sub-schema of an object type
/// - the schema is not in the `required` list of the object type
/// - the schema has an `allOf` definition
/// - the `allOf` definition possesses exactly one item
/// - the `allOf` item is a reference
///
/// `containing_object` must be the tuple with the object schema containing this schema, and the property name
/// referencing this schema.
fn is_property_singleton(
    spec: &OpenAPI,
    containing_object: ContainingObject,
    schema: &Schema,
) -> bool {
    // the schema is a property sub-schema of an object type
    let Some((object_type, property_name)) = containing_object else {
        return false;
    };
    if object_type
        .properties
        .get(property_name)
        .and_then(|schema_ref| Resolve::resolve(schema_ref, spec).ok())
        != Some(schema)
    {
        return false;
    }

    // the schema is not in the `required` list of the containing object type
    if object_type
        .required
        .iter()
        .any(|required| required == property_name)
    {
        return false;
    }

    // the schema has an `allOf` definition
    let SchemaKind::AllOf { all_of } = &schema.schema_kind else {
        return false;
    };

    // the `allOf` definition possesses exactly one item
    if all_of.len() != 1 {
        return false;
    }

    // the `allOf` item is a reference
    all_of[0].as_ref_str().is_some()
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
    /// The `content-type` MIME type known to be associated with this item.
    ///
    /// Not all items have a known MIME type. The most common case where this is set are
    /// direct `*Request` and `*Response` items.
    ///
    /// For items defined in `components/schemas` and defined inline within other item definitions,
    /// this should be unset.
    pub content_type: Option<String>,
    /// When true, we should `impl headers::Header` for this item.
    pub impl_header: bool,
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
            content_type: Default::default(),
            impl_header: Default::default(),
        }
    }
}

impl<R> Item<R>
where
    R: AsBackref + fmt::Debug,
{
    pub(crate) fn use_display_from_str(&self, model: &ApiModel<R>) -> Option<TokenStream> {
        let mut vd = self.value.use_display_from_str(model)?;
        if self.nullable {
            vd = quote!(Option<#vd>);
        }
        Some(vd)
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
            content_type,
            impl_header,
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
            content_type,
            impl_header,
        })
    }

    /// Parse a schema, recursively adding inline items
    ///
    /// `containing_object` is relevant only in the case that this schema is a property of an object type.
    /// In all other cases, it is fine to pass `None` for that parameter.
    pub(crate) fn parse_schema(
        spec: &OpenAPI,
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        schema: &Schema,
        containing_object: ContainingObject,
        content_type: Option<String>,
    ) -> Result<Self, ParseItemError> {
        let value: Value<Ref> = match &schema.schema_kind {
            SchemaKind::Type(Type::Boolean {}) => Value::Scalar(Scalar::Bool),
            SchemaKind::Type(Type::Number(number_type)) => number_type.try_into()?,
            SchemaKind::Type(Type::Integer(integer_type)) => integer_type.try_into()?,
            SchemaKind::Any(any_schema) => StringEnum::new(schema, any_schema)
                .map(Into::into)
                .unwrap_or(Scalar::Any.into()),
            SchemaKind::Type(Type::String(string_type)) => {
                Value::parse_string_type(string_type, &schema.schema_data)?
            }
            SchemaKind::Type(Type::Array(array_type)) => {
                Value::parse_array_type(spec, model, spec_name, rust_name, array_type)?
            }
            SchemaKind::Type(Type::Object(object_type)) => {
                Value::parse_object_type(spec, model, spec_name, rust_name, object_type)?
            }
            SchemaKind::OneOf { one_of } => {
                OneOfEnum::new(spec, model, spec_name, rust_name, schema, one_of)?.into()
            }
            SchemaKind::AllOf { all_of }
                if is_property_singleton(spec, containing_object, schema) =>
            {
                // `unwrap` and direct indexing are safe because `is_property_singleton` ensures we have
                // the right state for them.
                let reference = all_of[0].as_ref_str().unwrap();
                let ref_ = model
                    .get_named_reference(reference)
                    .map_err(|err| ParseItemError::AllOfSingleton(err.into()))?;
                PropertyOverride::new(schema, ref_).into()
            }
            SchemaKind::AllOf { .. } => return Err(ParseItemError::NonPropertyExtensionAllOf),
            SchemaKind::AnyOf { .. } | SchemaKind::Not { .. } => {
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
            content_type,
            impl_header: false,
        })
    }
}

impl Item {
    /// `true` when the item is a typedef.
    ///
    /// This disables derives when the item is emitted.
    pub(crate) fn is_typedef(&self) -> bool {
        self.newtype.is_none()
            && match &self.value {
                Value::Scalar(_)
                | Value::Set(_)
                | Value::List(_)
                | Value::Map(_)
                | Value::Ref(_)
                | Value::PropertyOverride(_) => true,
                Value::StringEnum(_) | Value::OneOfEnum(_) | Value::Object(_) => false,
            }
    }

    /// `true` when the item is probably json
    #[allow(dead_code)]
    pub(crate) fn is_json(&self) -> bool {
        // cast to bytes in case it's not ascii, so we don't have an indexing panic
        let content_type = self.content_type.as_deref().unwrap_or("json").as_bytes();
        // re-subslice the string to get the trailing four bytes
        let content_type = &content_type[content_type.len().checked_sub(4).unwrap_or_default()..];
        content_type.eq_ignore_ascii_case(b"json")
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

    /// If this item is trivial, emit its definition.
    pub(crate) fn trivial_definition(
        &self,
        model: &ApiModel,
    ) -> Result<Option<TokenStream>, UnknownReference> {
        if self.is_pub() {
            return Ok(None);
        }
        Ok(self.value.trivial_definition(model)?.map(|mut def| {
            if self.nullable {
                def = quote!(Option<#def>);
            }
            def
        }))
    }

    /// Generate an item definition for this item.
    ///
    /// The name resolver should be able to efficiently extract item names from references.
    ///
    /// If the item is not public and has a trivial value, nothing is emitted; implementations
    /// downstream should simply emit the trivial definition instead.
    pub fn emit<'a>(
        &self,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        if self.trivial_definition(model)?.is_some() {
            // This item is trivial and does not deserve its own definition.
            return Ok(Default::default());
        }

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

        let serde_as = self
            .value
            .use_serde_as_annotation(model)
            .then(|| quote!(#[openapi_gen::reexport::serde_with::serde_as(crate = "openapi_gen::reexport::serde_with")]));

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

        let serde_container_attributes = self.value.serde_container_attributes(self.is_typedef());
        let serde_container_attributes = (!serde_container_attributes.is_empty())
            .then(move || quote!(#[serde( #( #serde_container_attributes ),*)]));

        let pub_ = self.is_pub().then_some(quote!(pub));

        // the item definition is a multi-stage process:
        // we first compute it from the value, but then we
        // adjust it based on nullability and newtype options
        let mut item_def = self.value.emit_item_definition(model, name_resolver)?;

        // if this is nullable but we haven't defined a wrapper, then we need to make it nullable inline
        if self.nullable && wrapper_def.is_none() {
            item_def = quote!(Option<#item_def>);
        }

        // adjust the item definition based on newtype options
        if self.newtype.map(|options| options.pub_).unwrap_or_default() {
            item_def = quote!(pub #item_def);
        }
        if self.newtype.is_some() {
            item_def = quote!((#item_def));
        }

        let semicolon = (self.newtype.is_some() || self.is_typedef()).then_some(quote!(;));

        // in the future we may want custom derives here for subtypes, but for now, we're keeping things simple
        let canonical_form = self.newtype.is_some()
            .then(|| match &self.value {
                Value::Scalar(scalar) => Some(scalar.emit_type()),
                Value::Ref(ref_) | Value::PropertyOverride(PropertyOverride { ref_, .. }) => model.resolve(*ref_).ok().map(|item| {
                    let ident = make_ident(&item.rust_name);
                    quote!(#ident)
                }),
                _ => None
            })
            .flatten()
            .map(|inner_type| quote!(openapi_gen::newtype_derive_canonical_form!(#item_ident, #inner_type);));

        Ok(quote! {
            #wrapper_def

            #docs
            #serde_as
            #derives
            #serde_container_attributes
            #pub_ #item_keyword #item_ident #equals #item_def #semicolon

            #canonical_form
        })
    }

    /// The list of derives which should attach to this item.
    pub fn derives(&self, model: &ApiModel) -> Vec<TokenStream> {
        let mut derives = vec![quote!(Debug), quote!(Clone), quote!(PartialEq)];

        // extensible string enums require special de/serialization handling.
        // all other types can just derive standard serde stuff.
        match &self.value {
            Value::StringEnum(string_enum) if string_enum.extensible => {
                derives.push(quote!(
                    openapi_gen::reexport::serde_enum_str::Serialize_enum_str
                ));
                derives.push(quote!(
                    openapi_gen::reexport::serde_enum_str::Deserialize_enum_str
                ));
            }
            _ => {
                derives.push(quote!(openapi_gen::reexport::serde::Serialize));
                derives.push(quote!(openapi_gen::reexport::serde::Deserialize));
            }
        }

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
    #[error("`anyOf` and `not` schemas are not supported")]
    UnsupportedSchemaKind,
    #[error("this `allOf` schema did not meet the requirements of a property singleton")]
    NonPropertyExtensionAllOf,
    #[error("failed to get external documentation")]
    ExternalDocumentation(#[source] reqwest::Error),
    #[error("failed to construct `allOf` singleton")]
    AllOfSingleton(#[source] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum EmitError {
    #[error("unable to resolve name from reference: {0:?}")]
    UnresolvedReference(Reference),
}
