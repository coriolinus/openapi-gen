use std::collections::HashSet;

use heck::AsUpperCamelCase;
use openapiv3::{OpenAPI, Operation};
use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::{media_type, schema};

use super::make_ident;

pub fn get_ident(name: &str, url_fragment: &str, operation: &Operation) -> String {
    let fragment = operation
        .operation_id
        .clone()
        .unwrap_or_else(|| format!("{name} {url_fragment}"));
    format!("{}", AsUpperCamelCase(fragment))
}

/// Unwrap the contained expression or return a compile error.
macro_rules! unwrap_or_compile_error {
    ($e:expr, $m:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(err) => {
                let err_msg = format!("{}: {}", $m, err);
                return quote!(compile_error!(#err_msg));
            }
        }
    };
    (iter box $e:expr, $m:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(err) => {
                let err_msg = format!("{}: {}", $m, err);
                return Box::new(std::iter::once(quote!(compile_error!(#err_msg)))) as Box<dyn Iterator<Item = TokenStream>>;
            }
        }
    };
}

/// Make the request item for this operation.
///
/// Depending on the specification for this operation's request, this will
/// either be a type definition renaming some externally-provided type, or an
/// enum describing various accepted types according to the `content-type` of
/// the request body.
///
/// Note that the OpenAPI spec mandates:
///
/// > The `requestBody` is only supported in HTTP methods where the HTTP 1.1
/// > specification RFC7231 has explicitly defined semantics for request bodies.
/// > In other cases where the HTTP spec is vague, `requestBody` SHALL be
/// > ignored by consumers.
///
/// This function does not know what HTTP method is being used, and cannot obey
/// that mandate internally. It is the caller's responsibility to ensure that
/// this mandate is upheld.
pub fn make_request_item(spec: &OpenAPI, prefix_ident: &str, operation: &Operation) -> TokenStream {
    let item_name = make_ident(&format!("{}Request", prefix_ident,));

    let Some(request_body) = operation.request_body.as_ref() else {
        return quote!(pub type #item_name = (););
    };

    let request_body =
        unwrap_or_compile_error!(request_body.resolve(spec), "failed to resolve request body");

    let (option_open, option_close) = if !request_body.required {
        (quote!(Option<), quote!(>))
    } else {
        Default::default()
    };

    match media_type::distinct(&request_body.content) {
        media_type::Cardinality::Zero => quote!(pub type #item_name = ();),
        media_type::Cardinality::One(mime_type, media_type) => {
            let media_type_ident =
                make_ident(&media_type::get_ident(prefix_ident, mime_type, media_type));
            quote!(pub type #item_name = #option_open #media_type_ident #option_close;)
        }
        media_type::Cardinality::Several(iter) => {
            // define an enum over the various request types
            let variants = iter.map(|(mime_type, media_type)| {
                let variant_inner = media_type::get_ident(prefix_ident, mime_type, media_type);
                let variant_name = format!("{variant_inner}{}", AsUpperCamelCase(mime_type));
                let variant_inner = make_ident(&variant_inner);
                let variant_name = make_ident(&variant_name);

                quote!(#variant_name(#variant_inner))
            });
            let doc = request_body
                .description
                .as_ref()
                .map(|description| quote!(#[doc = #description]))
                .unwrap_or_default();
            if request_body.required {
                quote! {
                    #doc
                    pub enum #item_name {
                        #( #variants ),*
                    }
                }
            } else {
                let inner_name = make_ident(&format!("{item_name}Inner"));
                quote! {
                    #doc
                    pub enum #inner_name {
                        #( #variants ),*
                    }
                    pub type #item_name = Option<#inner_name>;
                }
            }
        }
    }
}

/// Convert a `StatusCode` enum into a `String` suitable for use as an ident for that code.
fn status_name(code: openapiv3::StatusCode) -> String {
    match code {
        openapiv3::StatusCode::Code(n) => http::StatusCode::from_u16(n)
            .ok()
            .and_then(|status| status.canonical_reason())
            .map(|reason| format!("{}", AsUpperCamelCase(reason)))
            .unwrap_or_else(|| format!("Code{n}")),
        openapiv3::StatusCode::Range(r) => match r {
            1 => "InformationalRange".into(),
            2 => "SuccessRange".into(),
            3 => "RedirectionRange".into(),
            4 => "ClientErrorRange".into(),
            5 => "ServerErrorRange".into(),
            _ => format!("Range{r}xx"),
        },
    }
}

/// Make the response item for this operation.
///
/// This is always an enum, even if it has only one variant. This is always an enum, even if it has zero variants!
/// Those are degenerate cases which do not receive special handling.
///
/// There is at least one variant for each defined HTTP status code which can be returned. If some status codes can
/// generate a variety of content types, then appropriate variants will be generated for those as well.
pub fn make_response_item(
    spec: &OpenAPI,
    prefix_ident: &str,
    operation: &Operation,
) -> TokenStream {
    let ident = make_ident(&format!("{prefix_ident}Response"));

    let response_iter = operation
        .responses
        .responses
        .iter()
        .map(|(status, response)| (status_name(status.clone()), response));
    let default_iter = operation
        .responses
        .default
        .as_ref()
        .map(|response| (String::from("Default"), response));
    let variants = response_iter
        .chain(default_iter)
        .flat_map(|(status_name, response)| {
            let response = unwrap_or_compile_error!(iter box
                response.resolve(spec),
                "failed to resolve response definition"
            );

            let doc = &response.description;

            // based on the cardinality of the response content, we adjust the variants we emit.
            match media_type::distinct(&response.content) {
                // If there is no response content, then there is a single variant with no suffix,
                // and no attached data.
                media_type::Cardinality::Zero => {
                    let variant_name = make_ident(&status_name);
                    Box::new(std::iter::once(quote!(#[doc = #doc] #variant_name)))
                        as Box<dyn Iterator<Item = TokenStream>>
                }
                // If there is one type of content, then there is a single variant with no suffix,
                // and appropriate attached data.
                media_type::Cardinality::One(mime_type, media_type) => {
                    let variant_name = make_ident(&status_name);
                    let variant_inner =
                        make_ident(&media_type::get_ident(prefix_ident, mime_type, media_type));
                    Box::new(std::iter::once(
                        quote!(#[doc = #doc] #variant_name(#variant_inner)),
                    ))
                }
                // If there is a variety of content, then each content type gets its own variant with
                // a suffix based on the media type of the content.
                media_type::Cardinality::Several(iter) => {
                    Box::new(iter.map(move |(mime_type, media_type)| {
                        let variant_name =
                            make_ident(&format!("{status_name}{}", AsUpperCamelCase(mime_type)));
                        let variant_inner =
                            make_ident(&media_type::get_ident(prefix_ident, mime_type, media_type));
                        quote!(#[doc = #doc] #variant_name(#variant_inner))
                    }))
                }
            }
        });

    quote! {
        pub enum #ident {#( #variants ),*}
    }
}

pub fn make_inline_request_item_definitions(
    spec: &OpenAPI,
    prefix_ident: &str,
    operation: &Operation,
) -> TokenStream {
    let Some(ref request_body) = operation.request_body else {
        return Default::default();
    };
    let request_body = unwrap_or_compile_error!(
        request_body.resolve(spec),
        "unable to resolve request body definition"
    );

    let mut already_generated = HashSet::new();

    let definitions = request_body
        .content
        .iter()
        .filter(|(_mime_type, media_type)| {
            // a reference, by definition, is defined elsewhere
            media_type::schema_ref(media_type).is_none()
        })
        .filter_map(|(mime_type, media_type)| {
            let item_name = media_type::get_ident(prefix_ident, mime_type, media_type);

            if !already_generated.insert(item_name.clone()) {
                // `insert` returns `false` if the set already contained this value
                // if the set already contains this value, don't regenerate
                return None;
            }

            let item_ident = make_ident(&item_name);

            Some(match media_type.schema.as_ref() {
                Some(schema) => {
                    let schema = schema.resolve(spec);
                    schema::make_items_for_schema(spec, &item_name, schema)
                }
                None => {
                    // if we have a type which doesn't define a schema, then
                    // the specification author believes that it is perfectly described
                    // by its mime type. In the future we can maybe do something clever
                    // by parsing the mime and switching on that, but for now, it's safe
                    // enough to just declare a typedef to `Vec<u8>`.

                    // there's no particular reason this typedef should be public; rustdoc
                    // will unify it appropriately.
                    quote!(type #item_ident = Vec<u8>;)
                }
            })
        });

    quote! {
        #(
            #definitions
        )*
    }
}

/// Make definitions for items defined inline within an operation.
///
/// Typically, operation definitions will just reference schemas defined elsewhere in the schema. However,
/// OpenAPI provides for the possibility of defining schemas inline, so we need to handle that case here.
pub fn make_inline_item_definitions(
    spec: &OpenAPI,
    prefix_ident: &str,
    operation: &Operation,
) -> TokenStream {
    todo!()
}
