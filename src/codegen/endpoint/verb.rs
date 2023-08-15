#[cfg(feature = "axum-support")]
use proc_macro2::TokenStream;

/// A HTTP Verb
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::Display, strum::EnumString)]
#[strum(serialize_all = "UPPERCASE", ascii_case_insensitive)]
pub enum Verb {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl Verb {
    pub(crate) fn request_body_is_legal(self) -> bool {
        match self {
            Verb::Get | Verb::Head | Verb::Delete | Verb::Trace => false,
            Verb::Put | Verb::Post | Verb::Options | Verb::Patch => true,
        }
    }

    #[cfg(feature = "axum-support")]
    pub(crate) fn emit_axum(&self) -> TokenStream {
        use quote::quote;

        use crate::codegen::make_ident;

        let method_name = self.to_string().to_lowercase();
        let method_ident = make_ident(&method_name);

        quote!(openapi_gen::reexport::axum::routing::#method_ident)
    }
}
