use crate::codegen::{
    api_model::{Ref, Reference, UnknownReference},
    make_ident,
};

use proc_macro2::TokenStream;
use quote::quote;

/// An inline definition of a mapping from String to T
#[derive(Debug, Clone)]
pub struct Map<Ref = Reference> {
    pub value_type: Option<Ref>,
}

impl Map<Ref> {
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
