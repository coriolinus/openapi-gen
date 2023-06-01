use super::{
    api_model::{Ref, Reference, UnknownReference},
    Item,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct List<Ref = Reference> {
    pub item: Ref,
}

impl List<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<List<Reference>, UnknownReference> {
        let Self { item } = self;
        let item = resolver(&item)?;
        Ok(List { item })
    }
}

// impl List {
//     pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
//         let item_name = format!("{derived_name}Item");
//         let item_referent = Item::reference_referent_ident(&self.item, &item_name);
//         quote!(Vec<#item_referent>)
//     }
// }
