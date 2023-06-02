use crate::codegen::make_ident;

use heck::AsUpperCamelCase;
use proc_macro2::TokenStream;
use quote::quote;

/// OpenAPI's string `enum` type
#[derive(Debug, Clone)]
pub struct StringEnum {
    pub variants: Vec<String>,
}

impl StringEnum {
    pub fn emit_definition(&self) -> TokenStream {
        let variants = self
            .variants
            .iter()
            .map(|variant| make_ident(&format!("{}", AsUpperCamelCase(variant))));
        quote! {
            { #( #variants ),* }
        }
    }
}
