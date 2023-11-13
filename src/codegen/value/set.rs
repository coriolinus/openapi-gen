use std::fmt;

use crate::{
    codegen::api_model::{AsBackref, Ref, Reference, UnknownReference},
    ApiModel,
};

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Set<Ref = Reference> {
    pub item: Ref,
}

impl<R> Set<R>
where
    R: AsBackref + fmt::Debug,
{
    pub(crate) fn use_serde_as_annotation(&self, model: &ApiModel<R>) -> bool {
        let Ok(item) = model.resolve(&self.item) else {
            return false;
        };
        item.serde_as_item_annotation(model).is_some()
    }

    pub(crate) fn serde_as_item_annotation(&self, model: &ApiModel<R>) -> Option<TokenStream> {
        let item = model.resolve(&self.item).ok()?;
        let inner = item.serde_as_item_annotation(model)?;
        Some(quote!(std::collections::HashSet<#inner>))
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
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let def = model.definition(self.item, name_resolver)?;
        Ok(quote!(std::collections::HashSet<#def>))
    }
}
