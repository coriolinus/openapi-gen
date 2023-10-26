//! Axum compatibility boilerplate generator.
//!
//! There are three major components to axum compatibility:
//!
//! - `impl IntoResponse` for all response types
//! - `impl Header` for all header types
//! - `fn build_router` to convert the implementation into an appropriate router
//!
//! For simplicity, we provide a single function `axum_items` which generates everything required.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    axum_compat::{header::impl_header, into_response::impl_into_response},
    codegen::{Scalar, Value},
    ApiModel,
};

mod build_router;
mod header;
mod into_response;

pub use into_response::default_response;

pub(crate) fn axum_items(model: &ApiModel) -> Result<TokenStream, Error> {
    let mut header_impls = Vec::new();

    for header_item in model.iter_items().filter_map(|ref_| {
        let item = model.resolve(ref_).ok()?;
        let has_existing_impl = matches!(&item.value, Value::Scalar(Scalar::AcceptHeader));
        (!has_existing_impl && item.impl_header).then_some(item)
    }) {
        header_impls.push(impl_header(model, header_item)?);
    }

    let mut into_response_impls = Vec::with_capacity(model.endpoints.len());
    for endpoint in model.endpoints.iter() {
        let reference = endpoint.response;

        into_response_impls.push(
            impl_into_response(model, &reference)
                .map_err(Error::context("implementing `IntoResponse`"))?,
        );
    }

    let build_router = build_router::fn_build_router(model)?;

    Ok(quote! {
        #( #header_impls )*
        #( #into_response_impls )*
        #build_router
    })
}

#[derive(Debug, thiserror::Error)]
#[error("{msg}")]
pub struct Error {
    msg: String,
    #[source]
    inner: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl Error {
    fn new(msg: impl Into<String>) -> Self {
        let msg = msg.into();
        let inner = None;
        Self { msg, inner }
    }

    fn context<C, E>(context: C) -> impl FnOnce(E) -> Self
    where
        C: Into<String>,
        Box<dyn 'static + std::error::Error + Send + Sync>: From<E>,
    {
        move |err| {
            let msg = context.into();
            let inner = Some(err.into());
            Self { msg, inner }
        }
    }
}
