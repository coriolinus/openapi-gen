use axum::{
    response::{IntoResponse, Response},
    Json,
};
use heck::{ToShoutySnakeCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Serialize;

use crate::{
    codegen::{
        endpoint::verb::Verb, make_ident, value::object::BODY_IDENT, Object, Reference,
        UnknownReference, Value,
    },
    ApiModel, AsStatusCode,
};

/// Construct a [`Response`][axum::response::Response] from the the default response type for an endpoint.
#[inline]
pub fn default_response<D>(default: D) -> Response
where
    D: AsStatusCode + Serialize,
{
    let status = default.as_status_code();
    (status, Json(default)).into_response()
}

/// `value` must evaluate to an `&str`
#[macro_export]
macro_rules! header_value_of {
    ($value:expr) => {
        match openapi_gen::reexport::http::HeaderValue::from_str($value) {
            Ok(value) => value,
            Err(_err) => return (
                openapi_gen::reexport::http::status::StatusCode::INTERNAL_SERVER_ERROR,
                format!("invalid header value for `{}` ({}:{}): all characters must be visible single-byte ascii (32-127)", stringify!($value), line!(), column!()),
            ).into_response(),
        }
    };
}

// impl IntoResponse for CreateNaturalPersonIdentificationResponse {
//     fn into_response(self) -> axum::response::Response {
//         match self {
//             CreateNaturalPersonIdentificationResponse::Created(created) => {
//                 let CreateNaturalPersonIdentificationResponseCreated { location, body } = created;
//                 let mut header_map = HeaderMap::with_capacity(1);
//                 let location = header_value_of!(&location);
//                 header_map.insert(LOCATION, location);
//                 (StatusCode::CREATED, header_map, Json(body)).into_response()
//             }
//             CreateNaturalPersonIdentificationResponse::Default(default) => {
//                 default_response(default)
//             }
//         }
//     }
// }

/// Implement `IntoResponse` for a response type.
///
/// This implementation handles extracting response headers and appropriate status codes from the response enum, which in turn
/// means that the response enum becomes a valid return value for a handler. This substantially simplifies handler generation.
pub(crate) fn impl_into_response(
    model: &ApiModel,
    response: &Reference,
    response_name: &str,
    variant_name: &str,
) -> Result<TokenStream, Error> {
    let item = model
        .resolve(*response)
        .ok_or_else(|| UnknownReference(format!("response reference: {response:?}")))
        .map_err(Error::context("getting response item"))?;

    let response_ident = make_ident(response_name);
    let variant_ident = make_ident(variant_name);
    let variant_binding = {
        let binding = variant_name.to_snake_case();
        make_ident(&binding)
    };
    let item_ident = make_ident(&item.rust_name);
    let status_ident = {
        let binding = variant_name.to_shouty_snake_case();
        make_ident(&binding)
    };

    // if the item is a default item, then we delegate to the `default_response` handler and trust that the
    // user has properly implemented the necessary traits
    if variant_name == "Default" {
        return Ok(quote! {
            #response_ident::#variant_ident(#variant_binding) => {
                openapi_gen::axum_compat::default_response(#variant_binding)
            }
        });
    }

    let mut headers = Vec::new();

    if let Some(content_type) = item.content_type.as_ref() {
        let key = quote!(openapi_gen::reexport::http::header::CONTENT_TYPE);
        let value = quote!(openapi_gen::reexport::http::HeaderValue::from_static(#content_type));
        headers.push((key, value));
    }

    if let Value::Object(Object {
        is_generated_body_and_headers: true,
        members,
    }) = &item.value
    {
        // unpack the object
        let member_idents = members.keys().map(|member| make_ident(member));
        let unpack_object = quote! {
            let #item_ident = { #( #member_idents ),* } = #variant_binding;
        };

        // transform headers
        for header_name in members.keys() {
            if header_name == BODY_IDENT {
                continue;
            }

            let header_ident = make_ident(header_name);

            let lower_name = header_name.to_lowercase();
            let key =
                quote!(openapi_gen::reexport::http::header::HeaderName::from_static(#lower_name));

            // TODO: invoke CanonicalForm::canonicalize here
            let value = quote!(openapi_gen::header_value_of!(&#header_ident));

            headers.push((key, value));
        }

        for (header_key_expr, header_value_expr) in headers {}
        todo!()
    }

    let define_header_map = (!headers.is_empty()).then(|| {
        let header_qty = headers.len();
        let header_insert = headers.iter().map(|(key, value)| {
            quote!{
                header_map.insert(#key, #value);
            }
        });
        quote!{
            let mut header_map = openapi_gen::reexport::http::header::HeaderMap::with_capacity(#header_qty);
            #( #header_insert )*
        }
    });
    let header_map_ident_comma = (!headers.is_empty()).then_some(quote!(header_map,));

    // todo: the rest of the owl
    let _ = quote! {
        (
            openapi_gen::reexport::http::status::StatusCode::#status_ident,
            #header_map_ident_comma
            body,
        ).into_response()
    };

    todo!()
}

#[derive(Debug, thiserror::Error)]
#[error("{context}")]
pub struct Error {
    context: String,
    #[source]
    inner: Box<dyn std::error::Error>,
}

impl Error {
    fn context<'a, E>(context: &'a str) -> impl 'a + Fn(E) -> Self
    where
        E: 'static + std::error::Error,
    {
        |err| {
            let context = context.into();
            let inner = Box::new(err) as _;
            Self { context, inner }
        }
    }
}
