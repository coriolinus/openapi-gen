use crate::codegen::make_ident;

use super::{maybe_map_reference_or, Item};
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
    pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
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
pub struct Variant {
    pub definition: ReferenceOr<Item>,
    pub mapping_name: Option<String>,
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
    /// - Else if the contained type is a reference, use the reference name after the last `/`.
    /// - Else if the contained type has a `name` field, use that.
    /// - Else use `Variant{idx:02}`.
    pub fn ident(&self, idx: usize) -> Ident {
        let derived_name = format!("Variant{idx:02}");
        let name = self
            .mapping_name
            .as_deref()
            .map(strip_slash_if_present)
            .or(self.definition.as_ref_str())
            .map(strip_slash_if_present)
            .map(ToOwned::to_owned)
            .or_else(|| {
                self.definition
                    .as_item()
                    .map(|item| item.referent_ident(&derived_name).to_string())
            })
            .unwrap_or(derived_name);
        let name = format!("{}", AsUpperCamelCase(name));
        make_ident(&name)
    }
}

/// OpenAPI's `oneOf` type
#[derive(Debug, Clone)]
pub struct OneOfEnum {
    pub discriminant: Option<String>,
    pub variants: Vec<Variant>,
}

impl OneOfEnum {
    pub fn try_from(
        schema_data: &SchemaData,
        variants: &[ReferenceOr<Schema>],
    ) -> Result<Self, String> {
        let discriminant = schema_data
            .discriminator
            .as_ref()
            .map(|discriminant| discriminant.property_name.clone());

        let variants = variants
            .iter()
            .map(|schema_ref| {
                let definition =
                    maybe_map_reference_or(schema_ref.as_ref(), |schema| schema.try_into())?;

                let mapping_name = schema_data
                    .discriminator
                    .as_ref()
                    .and_then(|discriminator| {
                        discriminator.mapping.iter().find_map(|(name, reference)| {
                            (Some(reference.as_str()) == schema_ref.as_ref_str())
                                .then(|| name.to_owned())
                        })
                    });

                Ok(Variant {
                    definition,
                    mapping_name,
                })
            })
            .collect::<Result<_, String>>()?;

        Ok(Self {
            discriminant,
            variants,
        })
    }

    pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
        let variants = self.variants.iter().enumerate().map(|(idx, variant)| {
            let ident = variant.ident(idx);
            let referent = Item::reference_referent_ident(&variant.definition, derived_name);
            quote!(#ident(#referent),)
        });
        quote! {
            {
                #( #variants )*
            }
        }
    }
}
