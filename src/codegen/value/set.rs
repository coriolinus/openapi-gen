use crate::{
    codegen::{
        api_model::{AsBackref, Ref, Reference, UnknownReference},
        make_ident,
    },
    ApiModel,
};

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Set<Ref = Reference> {
    pub item: Ref,
}

impl<R> Set<R> {
    pub(crate) fn use_serde_as_annotation(&self, model: &ApiModel<R>) -> bool
    where
        R: AsBackref,
    {
        let Some(item) = model.resolve_as_backref(&self.item) else {
            return false;
        };
        item.value.use_display_from_str(model)
    }
}

impl Set<Ref> {
    pub(crate) fn new(item: Ref) -> Self {
        Self { item }
    }

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
