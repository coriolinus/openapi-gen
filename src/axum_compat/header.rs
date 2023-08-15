use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    axum_compat::Error,
    codegen::{make_ident, Item},
    ApiModel,
};

/// Implement `Header` for a header type.
///
/// This is somewhat fragile. It assumes that it is safe to defer through `CanonicalForm`, which implies that that implementation
/// is infallible and produdes output containing only visible ASCII (32-127). We'll have to see if it causes problems in practice.
pub(crate) fn impl_header(_model: &ApiModel, item: &Item) -> Result<TokenStream, Error> {
    if item.is_typedef() {
        return Err(Error::new(format!(
            "item '{}' is a typdef, so cannot support a `Header` implementation",
            &item.rust_name
        )));
    }

    let item_name = make_ident(&item.rust_name);
    let header_name = item.spec_name.to_lowercase();

    Ok(quote! {
        impl openapi_gen::reexport::headers::Header for #item_name {
            fn name() -> &'static openapi_gen::reexport::headers::HeaderName {
                static NAME: openapi_gen::reexport::headers::HeaderName = openapi_gen::reexport::headers::HeaderName::from_static(#header_name);
                &NAME
            }

            fn decode<'i, I>(values: &mut I) -> Result<Self, openapi_gen::reexport::headers::Error>
            where
                Self: Sized,
                I: Iterator<Item = &'i openapi_gen::reexport::headers::HeaderValue>,
            {
                let value = values.next().ok_or_else(openapi_gen::reexport::headers::Error::invalid)?;
                let value_str = value.to_str().map_err(|_| openapi_gen::reexport::headers::Error::invalid())?;
                openapi_gen::CanonicalForm::validate(value_str).map_err(|_| openapi_gen::reexport::headers::Error::invalid())
            }

            fn encode<E>(&self, values: &mut E)
            where
                E: ::std::iter::Extend<openapi_gen::reexport::headers::HeaderValue>
            {
                let value = openapi_gen::CanonicalForm::canonicalize(self).expect("header encoding must be infallible");
                let header_value = openapi_gen::reexport::headers::HeaderValue::from_str(&value).expect("header canonical form must include only visible ascii");
                values.extend(::std::iter::once(header_value));
            }
        }
    })
}
