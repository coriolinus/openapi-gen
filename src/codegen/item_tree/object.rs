use crate::codegen::make_ident;

use super::{
    api_model::{Ref, Reference, UnknownReference},
    ApiModel,
};
use heck::AsSnakeCase;
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct ObjectMember<Ref = Reference> {
    pub definition: Ref,
    pub required: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl ObjectMember<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<ObjectMember<Reference>, UnknownReference> {
        let Self {
            definition,
            required,
            read_only,
            write_only,
        } = self;
        let definition = resolver(&definition)?;
        Ok(ObjectMember {
            definition,
            required,
            read_only,
            write_only,
        })
    }
}

impl ObjectMember {
    fn emit_definition<'a>(
        &self,
        member_name: &str,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let docs = model[self.definition]
            .docs
            .as_deref()
            .map(|docs| quote!(#[doc = #docs]));

        let snake_member_name = make_ident(&format!("{}", AsSnakeCase(member_name)));
        let item_ref = make_ident(name_resolver(self.definition)?);

        // todo: do we need this? shouldn't we be looking at a `Maybe` type already?
        let (option_head, option_tail) = if !self.required {
            (quote!(Option<), quote!(>))
        } else {
            Default::default()
        };

        Ok(quote! {
            #docs
            #snake_member_name: #option_head #item_ref #option_tail,
        })
    }
}

/// An inline definition of an object
#[derive(Debug, Clone)]
pub struct Object<Ref = Reference> {
    pub members: IndexMap<String, ObjectMember<Ref>>,
}

impl Object<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Object<Reference>, UnknownReference> {
        let Self { members } = self;
        let members = members
            .into_iter()
            .map(|(name, member)| member.resolve_refs(&resolver).map(|member| (name, member)))
            .collect::<Result<_, _>>()?;
        Ok(Object { members })
    }
}

impl Object {
    pub fn emit_definition<'a>(
        &self,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let members = self
            .members
            .iter()
            .map(|(member_name, member)| member.emit_definition(member_name, model, &name_resolver))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(quote! {
            {
                #( #members )*
            }
        })
    }
}
