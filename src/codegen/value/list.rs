use std::fmt;

use crate::{
    codegen::api_model::{AsBackref, Ref, Reference, UnknownReference},
    ApiModel,
};

use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct List<Ref = Reference> {
    pub item: Ref,
}

impl<R> List<R>
where
    R: AsBackref + fmt::Debug,
{
    pub(crate) fn use_serde_as_annotation(&self, model: &ApiModel<R>) -> bool {
        let Ok(item) = model.resolve(&self.item) else {
            return false;
        };
        item.use_display_from_str(model).is_some()
    }

    pub(crate) fn use_display_from_str(&self, model: &ApiModel<R>) -> Option<TokenStream> {
        let item = model.resolve(&self.item).ok()?;
        let inner = item.use_display_from_str(model)?;
        Some(quote!(Vec<#inner>))
    }
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
