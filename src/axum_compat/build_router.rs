use std::path::Path;

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::{endpoint::parameter::ParameterLocation, make_ident, Endpoint, UnknownReference},
    ApiModel,
};

use super::Error;

/// Convert the variable identifier from curly brackets to leading colons
///
/// ```ignore
/// # // we ignore this test because this is a private function in a private module; the test runner does not have visibility
/// # use openapi_gen::axum_compat::build_router::to_colon_path;
/// assert_eq!(
///     to_colon_path("/users/{id}"),
///     "/users/:id",
/// );
/// ```
fn to_colon_path(curly_bracket_path: &str) -> Result<String, Error> {
    let mut out = String::with_capacity(curly_bracket_path.len());
    let path = Path::new(curly_bracket_path);
    for component in path.components() {
        match component {
            std::path::Component::Prefix(_) => return Err(Error::new(format!("malformed path \"{curly_bracket_path}\": should be URL path not Windows"))),
            std::path::Component::CurDir |
            std::path::Component::ParentDir => return Err(Error::new(format!("malformed path \"{curly_bracket_path}\": current and parent components illegal"))),
            std::path::Component::RootDir => {
                // noop; the next component will handle this
            },
            std::path::Component::Normal(component) => {
                // always unconditionally prefix with the path separator
                out.push('/');

                let component = component.to_str().ok_or_else(|| Error::new(format!("malformed path \"{curly_bracket_path}\": non-String characters present")))?;
                if component.starts_with('{') && component.ends_with('}') {
                    out.push(':');
                    out.push_str(&component[1..component.len()-1]);
                } else {
                    out.push_str(component);
                }
            },
        }
    }

    Ok(out)
}

fn build_route(model: &ApiModel, endpoint: &Endpoint) -> Result<TokenStream, Error> {
    let path = to_colon_path(&endpoint.path)?;
    let verb = endpoint.verb.emit_axum();
    let prefix = quote!(openapi_gen::reexport::axum::extract);

    let mut parameters = Vec::new();
    let mut parameter_idents = Vec::new();
    let mut optional_parameter_map = Vec::new();

    for (key, param) in endpoint.parameters.iter() {
        let item = model
            .resolve(param.item_ref)
            .ok_or_else(|| UnknownReference(format!("{:?}", param.item_ref)))
            .map_err(Error::context(format!(
                "getting item for param \"{}\"",
                key.name
            )))?;

        let type_ident = make_ident(&item.rust_name);
        let variable_ident = make_ident(&item.rust_name.to_snake_case());

        let extractor = match key.location {
            Some(ParameterLocation::Header) => quote!(#prefix::TypedHeader),
            Some(ParameterLocation::Path) => quote!(#prefix::Path),
            Some(ParameterLocation::Query) => quote!(#prefix::Query),
            Some(ParameterLocation::Cookie) => {
                return Err(Error::new("cookie extractors not yet supported"))
            }
            None => {
                return Err(Error::new(format!(
                    "unknown parameter location for {}",
                    item.rust_name
                )))
            }
        };

        let (binding, bind_type) = if param.required {
            (
                quote!(#extractor(#variable_ident)),
                quote!(#extractor<#type_ident>),
            )
        } else {
            optional_parameter_map.push(quote! {
                let #variable_ident = #variable_ident.map(|#variable_ident| #variable_ident.0);
            });
            (
                quote!(#variable_ident),
                quote!(Option<#extractor<#type_ident>>),
            )
        };

        parameters.push(quote!(#binding: #bind_type));

        parameter_idents.push(variable_ident);
    }
    if let Some(ref_) = endpoint.request_body {
        // add body parameter last
        let item = model
            .resolve(ref_)
            .ok_or_else(|| UnknownReference(format!("{ref_:?}")))
            .map_err(Error::context("getting item for request body"))?;

        let type_ident = make_ident(&item.rust_name);
        let variable_ident = make_ident("request_body");

        if item.is_json() {
            parameters.push(quote! {
                #prefix::Json(#variable_ident): #prefix::Json<#type_ident>
            });
        } else {
            parameters.push(quote! {
                #variable_ident: Vec<u8>
            });
        }

        parameter_idents.push(variable_ident);
    }

    let method_name = make_ident(&endpoint.function_name(None));

    Ok(quote! {
        .route(
            #path,
            #verb({
                let instance = instance.clone();
                move |#( #parameters ),*| async move {
                    #( #optional_parameter_map )*
                    instance.#method_name(#( #parameter_idents ),*).await
                }
            })
        )
    })
}

/// Create `fn build_router`, which transforms an arbitrary `Api` instance into a `Router`.
pub(crate) fn fn_build_router(model: &ApiModel) -> Result<TokenStream, Error> {
    let mut routes = Vec::<TokenStream>::new();

    for endpoint in model.endpoints.iter() {
        let route = build_route(model, endpoint)?;
        routes.push(route);
    }

    Ok(quote! {
        /// Transform an instance of [`trait Api`][Api] into a [`Router`][axum::Router].
        pub fn build_router<Instance>(instance: Instance) -> openapi_gen::reexport::axum::Router
        where
            Instance: 'static + Api + Send + Sync
        {
            // `instance` is unused if there are no endpoints
            #[allow(unused_variables)]
            let instance = ::std::sync::Arc::new(instance);
            openapi_gen::reexport::axum::Router::new()
            #( #routes )*
        }
    })
}
