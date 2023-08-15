use axum::{
    response::{IntoResponse, Response},
    Json,
};
use heck::{ToShoutySnakeCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Serialize;

use crate::{
    axum_compat::Error,
    codegen::{make_ident, value::object::BODY_IDENT, Object, Reference, UnknownReference, Value},
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

#[macro_export]
macro_rules! or_ice {
    ($value:expr) => {
        match $value {
            Ok(value) => value,
            Err(err) => {
                return (
                    openapi_gen::reexport::http::status::StatusCode::INTERNAL_SERVER_ERROR,
                    format!(
                        "invalid header value for `{}` ({}:{}): {err}",
                        stringify!($value),
                        line!(),
                        column!()
                    ),
                )
                    .into_response()
            }
        }
    };
}

/// `value` must implement `CanonicalForm<JsonRepresentation=String>`.
#[macro_export]
macro_rules! header_value_of {
    ($value:expr) => {{
        let value = $crate::or_ice!(openapi_gen::CanonicalForm::canonicalize($value));
        $crate::or_ice!(openapi_gen::reexport::http::HeaderValue::from_str(&value))
    }};
}

/// Implement a single `match` arm of `IntoResponse`.
///
/// This implementation handles extracting response headers and appropriate status codes from the response enum, which in turn
/// means that the response enum becomes a valid return value for a handler. This substantially simplifies handler generation.
fn impl_into_response_for_response_type(
    model: &ApiModel,
    response: &Reference,
    response_name: &str,
    variant_name: &str,
) -> Result<TokenStream, Error> {
    let item = model
        .resolve(*response)
        .ok_or_else(|| {
            UnknownReference(format!(
                "response variant reference: {response:?} ({response_name})"
            ))
        })
        .map_err(Error::context("getting response variant item"))?;

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

    let mut unpack_object = None;
    let body;

    if let Value::Object(Object {
        is_generated_body_and_headers: true,
        members,
    }) = &item.value
    {
        // the identifier for the body is constant in this case
        body = make_ident("body");

        // unpack the object
        let member_idents = members.keys().map(|member| make_ident(member));
        unpack_object = Some(quote! {
            let #item_ident { #( #member_idents ),* } = #variant_binding;
        });

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
    } else {
        // don't panic, this was just a non-object value which got passed through because there were no headers
        // all we need to do here is ensure `body` is defined
        body = variant_binding.clone();
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

    // wrap the body in a JSON wrapper if it is not a unit type, and
    // the item content-type ends with "json"
    let wrap_with_json = !matches!(&item.value, Value::Scalar(crate::codegen::Scalar::Unit))
        && item
            .content_type
            .as_deref()
            .unwrap_or("json")
            .eq_ignore_ascii_case("json");

    let body = if wrap_with_json {
        quote!(openapi_gen::reexport::axum::Json(#body))
    } else {
        quote!(#body)
    };

    let into_response = quote! {
        (
            openapi_gen::reexport::http::status::StatusCode::#status_ident,
            #header_map_ident_comma
            #body,
        ).into_response()
    };

    Ok(quote! {
        #response_ident::#variant_ident(#variant_binding) => {
            #unpack_object
            #define_header_map
            #into_response
        }
    })
}

/// Implement `IntoResponse` for a response type.
///
/// This implementation handles extracting response headers and appropriate status codes from the response enum, which in turn
/// means that the response enum becomes a valid return value for a handler. This substantially simplifies handler generation.
pub(crate) fn impl_into_response(
    model: &ApiModel,
    response_enum: &Reference,
) -> Result<TokenStream, Error> {
    let item = model
        .resolve(*response_enum)
        .ok_or_else(|| UnknownReference(format!("response reference: {response_enum:?}")))
        .map_err(Error::context("getting response item"))?;

    let response_name = &item.rust_name;
    let response_ident = make_ident(response_name);

    let Value::OneOfEnum(oo_enum) = &item.value else {
        let err = Error::new(format!("response reference: {response_enum:?} ({response_name})"));
        return Err(Error::context("expected response variant to be OneOfEnum")(err));
    };

    let mut branches = Vec::new();

    for variant in &oo_enum.variants {
        let variant_name = variant.computed_name().ok_or_else(|| {
            let err = Error::new("failed to get computed variant name for response variant");
            Error::context("computing branches within `impl_into_response`")(err)
        })?;

        branches.push(impl_into_response_for_response_type(
            model,
            &variant.definition,
            response_name,
            variant_name,
        )?);
    }

    Ok(quote! {
        impl openapi_gen::reexport::axum::response::IntoResponse for #response_ident {
            fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
                match self {
                    #( #branches )*
                }
            }
        }
    })
}
