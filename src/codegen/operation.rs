use std::collections::HashSet;

use heck::AsUpperCamelCase;
use openapiv3::{OpenAPI, Operation};
use quote::quote;

use crate::codegen::media_type;

use super::make_ident;

pub fn get_ident(name: &str, url_fragment: &str, operation: &Operation) -> String {
    let fragment = operation
        .operation_id
        .clone()
        .unwrap_or_else(|| format!("{name} {url_fragment}"));
    format!("{}", AsUpperCamelCase(fragment))
}

pub fn make_request_item(
    spec: &OpenAPI,
    operation_name: &str,
    operation_url_fragment: &str,
    operation: &Operation,
) -> proc_macro2::TokenStream {
    let prefix_ident = get_ident(operation_name, operation_url_fragment, operation);

    let item_name = make_ident(&format!("{}Request", prefix_ident,));

    let Some(request_body) = operation.request_body.as_ref() else {
        return quote!(pub type #item_name = (););
    };

    let request_body = match request_body.resolve(spec) {
        Ok(request_body) => request_body,
        Err(err) => {
            let err_msg = format!("failed to resolve request body: {err}");
            return quote! {
                compile_error!(#err_msg);
            };
        }
    };

    let (option_open, option_close) = if !request_body.required {
        (quote!(Option<), quote!(>))
    } else {
        Default::default()
    };

    // we don't actually care how many different content types the body can accept; what we care about are distinct schemas
    // unfortunately, the `Schema` type doesn't implement `Hash` or `Ord`, so we can't compare the actual structs in a set directly.
    //
    let n_distinct_schema_refs = request_body
        .content
        .values()
        .filter_map(|media_type| {
            media_type
                .schema
                .as_ref()
                .and_then(|schemaref| schemaref.as_ref_str())
        })
        .collect::<HashSet<_>>()
        .len();
    let n_distinct_local_schemas = request_body
        .content
        .values()
        .filter(|media_type| {
            media_type
                .schema
                .as_ref()
                .map(|schemaref| schemaref.as_ref_str().is_none())
                .unwrap_or_default()
        })
        .count();
    // What we actually want to do depends on both counts.
    match (n_distinct_local_schemas, n_distinct_schema_refs) {
        // if there are no schemas at all, the request type is the unit type
        (0, 0) => quote!(pub type #item_name = ();),
        // if there is one distinct ref and no local types, or no distinct refs adn one local type, then
        // we can just defer directly to that type
        (0, 1) | (1, 0) => {
            // define a typedef for the contained request type
            let (mime_type, media_type) = request_body.content.first().unwrap();
            let media_type_ident =
                make_ident(&media_type::get_ident(&prefix_ident, mime_type, media_type));
            quote!(pub type #item_name = #option_open #media_type_ident #option_close;)
        }
        // in all other cases we have to be explicit about what we mean, because we can't tell if two schemas are the same
        _ => {
            // define an enum over the various request types
            let variants = request_body.content.iter().map(|(mime_type, media_type)| {
                let variant_inner = media_type::get_ident(&prefix_ident, mime_type, media_type);
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
