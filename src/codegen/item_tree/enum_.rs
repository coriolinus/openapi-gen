use crate::codegen::make_ident;

use super::{
    api_model::{AsBackref, Ref, Reference, UnknownReference},
    maybe_map_reference_or, ApiModel, Item,
};
use heck::AsUpperCamelCase;
use openapiv3::{ReferenceOr, Schema, SchemaData};
use proc_macro2::{Ident, TokenStream};
use quote::quote;

/// OpenAPI's string `enum` type
#[derive(Debug, Clone)]
pub struct StringEnum {
    pub variants: Vec<String>,
}

impl StringEnum {
    pub fn emit_definition(&self, _derived_name: &str) -> TokenStream {
        let variants = self
            .variants
            .iter()
            .map(|variant| make_ident(&format!("{}", AsUpperCamelCase(variant))));
        quote! {
            { #( #variants ),* }
        }
    }
}

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

impl<R> Variant<R> {
    /// Compute an appropriate variant identifier for this variant.
    ///
    /// `idx` should be the index of this variant among all variants in the enum.
    ///
    /// Rules:
    ///
    /// - If there is an explicit mapping, use the portion of the mapping name after the last `/`.
    /// - Else use the variant's inner identifier
    /// - Else use `Variant{idx:02}`.
    pub(crate) fn compute_variant_name(&self, model: &ApiModel<R>, idx: usize) -> String
    where
        R: AsBackref,
    {
        let name = self
            .mapping_name
            .as_deref()
            .map(strip_slash_if_present)
            .or_else(|| model.find_name_for_reference(&self.definition))
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("Variant{idx:02}"));
        format!("{}", AsUpperCamelCase(name))
    }
}

/// OpenAPI's `oneOf` type
#[derive(Debug, Clone)]
pub struct OneOfEnum<Ref = Reference> {
    pub discriminant: Option<String>,
    pub variants: Vec<Variant<Ref>>,
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

//     pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
//         let variants = self.variants.iter().enumerate().map(|(idx, variant)| {
//             let ident = variant.ident(idx);
//             let name = format!("{derived_name}Variant{idx}");
//             let referent = Item::reference_referent_ident(&variant.definition, &name);
//             quote!(#ident(#referent),)
//         });
//         quote! {
//             {
//                 #( #variants )*
//             }
//         }
//     }
// }
