use crate::codegen::{
    api_model::{Ref, Reference, UnknownReference},
    make_ident, ApiModel,
};

use heck::AsSnakeCase;
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct ObjectMember<Ref = Reference> {
    pub definition: Ref,
    pub read_only: bool,
    pub write_only: bool,
    pub inline_option: bool,
}

impl ObjectMember<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<ObjectMember<Reference>, UnknownReference> {
        let Self {
            definition,
            read_only,
            write_only,
            inline_option,
        } = self;
        let definition = resolver(&definition)?;
        Ok(ObjectMember {
            definition,
            read_only,
            write_only,
            inline_option,
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

        let mut serde_attributes = Vec::new();

        let mut snake_member_name = format!("{}", AsSnakeCase(member_name));
        model.deconflict_member_or_variant_ident(&mut snake_member_name);
        let snake_member_name = make_ident(&snake_member_name);
        let item_ref = make_ident(name_resolver(self.definition)?);

        if snake_member_name != member_name {
            serde_attributes.push(quote!(rename = #member_name));
        }

        // `self.inline_option` is set when this item is optional, not intrinsically,
        // but within the context of this object.
        let mut item_ref = quote!(#item_ref);
        if self.inline_option {
            item_ref = quote!(Option<#item_ref>);
            serde_attributes.push(quote!(skip_serializing_if = "Option::is_none"));
        }

        let serde_attributes = (!serde_attributes.is_empty()).then(|| {
            quote! {
                #[serde(#( #serde_attributes ),*)]
            }
        });

        Ok(quote! {
            #docs
            #serde_attributes
            pub #snake_member_name: #item_ref,
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
