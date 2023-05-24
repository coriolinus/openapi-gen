use super::Item;
use openapiv3::ReferenceOr;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct List {
    pub item: Box<ReferenceOr<Item>>,
}

impl List {
    pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
        let item_referent = Item::reference_referent_ident(&self.item, derived_name);
        quote!(Vec<#item_referent>)
    }
}
