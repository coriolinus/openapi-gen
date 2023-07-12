use crate::codegen::make_ident;

use super::api_model::{Ref, Reference, UnknownReference};
use heck::AsUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct Variant<Ref = Reference> {
    pub definition: Ref,
    pub mapping_name: Option<String>,
}

impl Variant<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Variant<Reference>, UnknownReference> {
        let Self {
            definition,
            mapping_name,
        } = self;
        let definition = resolver(&definition)?;
        Ok(Variant {
            definition,
            mapping_name,
        })
    }
}

/// Get the part of the contained value after the last slash, or the whole thing if no slashes are present.
fn strip_slash_if_present(v: &str) -> &str {
    v.rsplit('/').next().unwrap_or(v)
}

impl Variant {
    /// Compute an appropriate variant identifier for this variant.
    ///
    /// `idx` should be the index of this variant among all variants in the enum.
    ///
    /// Rules:
    ///
    /// - If there is an explicit mapping, use the portion of the mapping name after the last `/`.
    /// - Else use the variant's identifier
    /// - Else use `Variant{idx:02}`.
    fn compute_variant_name<'a>(
        &self,
        idx: usize,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> String {
        self.mapping_name
            .as_deref()
            .map(strip_slash_if_present)
            .or_else(|| name_resolver(self.definition).ok())
            .map(|name| format!("{}", AsUpperCamelCase(name)))
            .unwrap_or_else(|| format!("Variant{idx:02}"))
    }
}

/// OpenAPI's `oneOf` type
#[derive(Debug, Clone)]
pub struct OneOfEnum<Ref = Reference> {
    pub discriminant: Option<String>,
    pub variants: Vec<Variant<Ref>>,
}

impl<R> Default for OneOfEnum<R> {
    fn default() -> Self {
        Self {
            discriminant: Default::default(),
            variants: Default::default(),
        }
    }
}

impl OneOfEnum<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<OneOfEnum<Reference>, UnknownReference> {
        let Self {
            discriminant,
            variants,
        } = self;
        let variants = variants
            .into_iter()
            .map(|variant| variant.resolve_refs(&resolver))
            .collect::<Result<_, _>>()?;
        Ok(OneOfEnum {
            discriminant,
            variants,
        })
    }
}

impl OneOfEnum {
    pub fn emit_definition<'a>(
        &self,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let variants = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, variant)| {
                let ident = make_ident(&variant.compute_variant_name(idx, &name_resolver));
                let referent = make_ident(name_resolver(variant.definition)?);
                Ok(quote!(#ident(#referent),))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(quote! {
            {
                #( #variants )*
            }
        })
    }

    pub(crate) fn serde_container_attributes(&self) -> Vec<TokenStream> {
        let mut attributes = Vec::new();
        if let Some(tag) = &self.discriminant {
            attributes.push(quote!(tag = #tag));
        } else {
            attributes.push(quote!(untagged));
        }
        attributes
    }
}
