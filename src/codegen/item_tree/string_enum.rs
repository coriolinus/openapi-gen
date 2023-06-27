use crate::codegen::make_ident;

use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;

/// OpenAPI's string `enum` type.
///
/// Also covers `x-extensible-enum`.
#[derive(Debug, Clone)]
pub struct StringEnum {
    pub variants: Vec<String>,
    pub extensible: bool,
}

impl StringEnum {
    pub fn emit_definition(&self) -> TokenStream {
        let mut variants = self
            .variants
            .iter()
            .map(|variant| {
                let ident = make_ident(&variant.to_upper_camel_case());
                quote!(#ident)
            })
            .collect::<Vec<_>>();
        if self.extensible {
            // Normally, we're just going to call the "other" field "Other", but
            // in case there's a conflict, ensure it's unique. We're not too
            // concerned about efficiency here; this should only very rarely loop
            // more than once or twice.
            let mut other_name = "Other".to_string();
            while self.variants.contains(&other_name) {
                other_name.push('_');
            }
            let other_name = make_ident(&other_name);
            variants.push(quote!(#other_name(String)));
        }
        quote! {
            { #( #variants ),* }
        }
    }

    pub(crate) fn impls_copy(&self) -> bool {
        !self.extensible
    }
}
