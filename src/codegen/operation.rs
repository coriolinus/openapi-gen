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

    match request_body.content.len() {
        0 => quote!(pub type #item_name = ();),
        1 => {
            // define a typedef for the contained request type
            let (mime_type, media_type) = request_body.content.first().unwrap();
            let media_type_ident =
                make_ident(&media_type::get_ident(&prefix_ident, mime_type, media_type));
            quote!(pub type #item_name = #media_type_ident;)
        }
        _ => {
            // define an enum over the various request types
            let variants = request_body.content.iter().map(|(mime_type, media_type)| {
                let variant =
                    make_ident(&media_type::get_ident(&prefix_ident, mime_type, media_type));
                quote!(#variant(#variant))
            });
            quote! {
                pub enum #item_name {
                    #( #variants ),*
                }
            }
        }
    }
}
