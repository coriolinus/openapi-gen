use super::Item;
use openapiv3::ReferenceOr;
use proc_macro2::TokenStream;
use quote::quote;

/// An inline definition of a mapping from String to T
#[derive(Debug, Clone)]
pub struct Map {
    pub value_type: Option<Box<ReferenceOr<Item>>>,
}

impl Map {
    pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
        let value_referent = self
            .value_type
            .as_ref()
            .map(|vt| {
                let vt_ident = Item::reference_referent_ident(vt, derived_name);
                quote!(#vt_ident)
            })
            .unwrap_or(quote!(serde_json::Value));
        quote!(std::collections::HashMap<String, #value_referent>)
    }
}
