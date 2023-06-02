use crate::codegen::make_ident;

use super::api_model::{Ref, Reference, UnknownReference};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Set<Ref = Reference> {
    pub item: Ref,
}

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

impl Set {
    pub fn emit_definition<'a>(
        &self,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let item_name = name_resolver(self.item)?;
        let ident = make_ident(item_name);
        Ok(quote!(std::collections::HashSet<#ident>))
    }
}
