use crate::{
    codegen::{
        api_model::{Ref, Reference, UnknownReference},
        make_ident,
    },
    ApiModel,
};

use openapiv3::{AdditionalProperties, OpenAPI};
use proc_macro2::TokenStream;
use quote::quote;

use super::ValueConversionError;

/// An inline definition of a mapping from String to T
#[derive(Debug, Clone)]
pub struct Map<Ref = Reference> {
    pub value_type: Option<Ref>,
}

impl Map<Ref> {
    pub(crate) fn new(
        spec: &OpenAPI,
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        additional_properties: &AdditionalProperties,
    ) -> Result<Self, ValueConversionError> {
        let value_type = match additional_properties {
            openapiv3::AdditionalProperties::Any(_) => None,
            openapiv3::AdditionalProperties::Schema(schema_ref) => {
                let rust_name = format!("{rust_name}Item");
                let item = model
                    .convert_reference_or(
                        spec,
                        spec_name,
                        &rust_name,
                        None,
                        &schema_ref.as_ref().as_ref(),
                        None,
                    )
                    .map_err(ValueConversionError::from_inline(&rust_name))?;
                Some(item)
            }
        };
        Ok(Map { value_type })
    }

    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Map<Reference>, UnknownReference> {
        let Self { value_type } = self;
        let value_type = value_type.map(|ref_| resolver(&ref_)).transpose()?;
        Ok(Map { value_type })
    }
}

impl Map {
    pub fn emit_definition<'a>(
        &self,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let value_referent = self
            .value_type
            .map(|reference| {
                let item_name = name_resolver(reference)?;
                let ident = make_ident(item_name);
                Ok(quote!(#ident))
            })
            .transpose()?
            .unwrap_or(quote!(openapi_gen::reexport::serde_json::Value));
        Ok(quote!(std::collections::HashMap<String, #value_referent>))
    }
}
