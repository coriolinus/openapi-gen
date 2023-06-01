use super::{
    api_model::{Ref, Reference, UnknownReference},
    Item,
};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Set<Ref = Reference> {
    pub item: Ref,
}

// impl Set {
//     pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
//         let item_referent = Item::reference_referent_ident(&self.item, derived_name);
//         quote!(Vec<#item_referent>)
//     }
// }

impl Set<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Set<Reference>, UnknownReference> {
        let Self { item } = self;
        let item = resolver(&item)?;
        Ok(Set { item })
    }
}
