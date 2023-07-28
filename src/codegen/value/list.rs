use crate::codegen::{
    api_model::{Ref, Reference, UnknownReference},
    make_ident,
};

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct List<Ref = Reference> {
    pub item: Ref,
}

impl List<Ref> {
    pub(crate) fn new(item: Ref) -> Self {
        Self { item }
    }

    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<List<Reference>, UnknownReference> {
        let Self { item } = self;
        let item = resolver(&item)?;
        Ok(List { item })
    }
}

impl List {
    pub fn emit_definition<'a>(
        &self,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let item_name = name_resolver(self.item)?;
        let ident = make_ident(item_name);
        Ok(quote!(Vec<#ident>))
    }
}
