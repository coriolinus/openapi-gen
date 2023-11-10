use crate::{
    codegen::api_model::{Ref, Reference, UnknownReference},
    ApiModel,
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
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let def = model.definition(self.item, name_resolver)?;
        Ok(quote!(Vec<#def>))
    }
}
