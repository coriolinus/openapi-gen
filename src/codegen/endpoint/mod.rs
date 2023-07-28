use std::collections::HashMap;

use heck::{AsUpperCamelCase, ToSnakeCase};
use indexmap::IndexMap;
use openapiv3::OpenAPI;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::{
        endpoint::parameter::ParameterLocation, make_ident, Ref, Reference, UnknownReference,
    },
    openapi_compat::path_items,
    ApiModel,
};

pub(crate) mod parameter;
use parameter::{convert_param_ref, Parameter, ParameterKey};

pub(crate) mod request_body;

pub(crate) mod response;

pub(crate) mod verb;
use verb::Verb;

#[derive(Debug, Clone)]
pub struct Endpoint<Ref = Reference> {
    /// Relative path from the server URL to this endpoint.
    ///
    /// May contain parameters with `{name}` notation.
    pub path: String,
    /// Endpoint-level documentation.
    ///
    /// Preferentially taken from `PathItem::description`, falling back to `PathItem::summary` if required.
    pub endpoint_documentation: Option<String>,
    /// Operation-level endpoint documentation.
    ///
    /// If external documentation is specified, that is queried first. Otherwise, this falls back first to
    /// `Operation::description` and then to `Operation::summary`.
    pub operation_documentation: Option<String>,
    pub verb: Verb,
    /// The parameters are derived first from `PathItem::parameters`, then updated from `Operation::parameters`.
    pub parameters: IndexMap<ParameterKey, Parameter<Ref>>,
    /// Operation ID.
    ///
    /// If set, this is used as the basis for the rust name.
    pub operation_id: Option<String>,
    /// Request body.
    ///
    /// This is overridden to `None` if `verb` is `GET`, `HEAD`, `DELETE`, or `TRACE`.
    pub request_body: Option<Ref>,
    /// Response body enum.
    ///
    /// This is always an enum, even in the event that there are 0 variants. (That is a degenerate case
    /// indicating a malformed OpenAPI specification).
    pub response: Ref,
}

impl<R> Endpoint<R> {
    /// Compute a documentation string for this endpoint.
    fn doc_string(&self) -> String {
        let internal_docs = match (
            self.endpoint_documentation.as_deref(),
            self.operation_documentation.as_deref(),
        ) {
            // we put operation docs before endpoint docs because they should in theory be more specific
            (Some(endpoint), Some(operation)) => Some(format!("{operation}\n\n{endpoint}\n\n")),
            (Some(docs), None) | (None, Some(docs)) => Some(docs.to_owned()),
            (None, None) => None,
        };

        let mut docs = internal_docs
            .map(|internal| format!("{internal}\n\n## Endpoint Data\n\n"))
            .unwrap_or_default();
        docs.push_str(&format!("`{} {}`\n\n", self.verb, self.path));
        if let Some(operation_id) = self.operation_id.as_deref() {
            docs.push_str("Operation ID: `");
            docs.push_str(operation_id);
            docs.push_str("`\n\n");
        }

        docs
    }

    /// Compute the function name for this endpoint.
    ///
    /// The endpoint doesn't know internally whether it is unique at the verb/path combo,
    /// or whether it needs a suffix to disambiguate from other content types. That must
    /// therefore be provided externally.
    fn function_name(&self, suffix: Option<&str>) -> String {
        self.operation_id
            .clone()
            .unwrap_or_else(|| {
                let mut name = format!("{} {}", self.verb, self.path);
                if let Some(suffix) = suffix {
                    name.push(' ');
                    name.push_str(suffix);
                }
                name
            })
            .to_snake_case()
    }

    /// Compute the function parameters.
    ///
    /// Items are `(name, type, required)` where `name` is an appropriate parameter name, and `type` is convertable into a type ident.
    /// `required` is `true` when the item is mandatory.
    fn function_parameters(&self) -> impl Iterator<Item = (String, R, bool)>
    where
        R: Clone,
    {
        struct ByName<R2> {
            no_location: Option<(R2, bool)>,
            by_location: HashMap<ParameterLocation, (R2, bool)>,
        }

        impl<R2> Default for ByName<R2> {
            fn default() -> Self {
                Self {
                    no_location: Default::default(),
                    by_location: Default::default(),
                }
            }
        }

        impl<R2> ByName<R2> {
            fn len(&self) -> usize {
                self.by_location.len() + if self.no_location.is_some() { 1 } else { 0 }
            }

            fn into_iter(self) -> impl Iterator<Item = (Option<ParameterLocation>, R2, bool)> {
                self.by_location
                    .into_iter()
                    .map(|(location, (ref_, required))| (Some(location), ref_, required))
                    .chain(
                        self.no_location
                            .into_iter()
                            .map(|(ref_, required)| (None, ref_, required)),
                    )
            }
        }

        let mut params_by_name = HashMap::<_, ByName<R>>::new();
        for (ParameterKey { name, location }, Parameter { required, item_ref }) in
            self.parameters.iter()
        {
            let by_name = params_by_name.entry(name.clone()).or_default();
            match location {
                None => by_name.no_location = Some((item_ref.clone(), *required)),
                Some(location) => {
                    by_name
                        .by_location
                        .insert(*location, (item_ref.clone(), *required));
                }
            }
        }

        params_by_name.into_iter().flat_map(|(name, by_name)| {
            let append_location_name = by_name.len() != 1;
            by_name
                .into_iter()
                .map(move |(maybe_location, ref_, required)| {
                    let name = if append_location_name {
                        let location_name = maybe_location
                            .map(|location| location.to_string())
                            .unwrap_or_else(|| "Unknown".into());
                        format!("{name} {location_name}")
                    } else {
                        name.clone()
                    }
                    .to_snake_case();

                    (name, ref_, required)
                })
        })
    }
}

impl Endpoint<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Endpoint<Reference>, UnknownReference> {
        let Self {
            path,
            endpoint_documentation,
            operation_documentation,
            verb,
            parameters,
            operation_id,
            request_body,
            response,
        } = self;

        let parameters = parameters
            .into_iter()
            .map(|(pkey, param)| Ok((pkey, param.resolve_refs(&resolver)?)))
            .collect::<Result<_, _>>()?;
        let request_body = request_body
            .map(|request_body_ref| resolver(&request_body_ref))
            .transpose()?;
        let response = resolver(&response)?;

        Ok(Endpoint {
            path,
            endpoint_documentation,
            operation_documentation,
            verb,
            parameters,
            operation_id,
            request_body,
            response,
        })
    }
}

impl Endpoint {
    /// Generate an item definition for this item.
    ///
    /// The name resolver should be able to efficiently extract item names from references.
    pub fn emit<'a>(
        &self,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let docs = self.doc_string();
        let docs = quote!(#[doc = #docs]);
        // todo: proper suffix
        let function_name = make_ident(&self.function_name(None));

        let parameters = self
            .function_parameters()
            .map(|(name, ref_, required)| {
                let param_name = make_ident(&name);
                let type_name = make_ident(name_resolver(ref_)?);
                let mut type_name = quote!(#type_name);
                if !required {
                    type_name = quote!(Option< #type_name >);
                }
                Ok(quote!(#param_name: #type_name))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let request_body = self
            .request_body
            .map(|ref_| {
                let type_name = name_resolver(ref_)?;
                let type_name = make_ident(type_name);

                Ok(quote!(request_body: #type_name))
            })
            .transpose()?;

        let response_body = make_ident(name_resolver(self.response)?);

        Ok(quote! {
            #docs
            async fn #function_name (
                #(
                    #parameters,
                )*
                #request_body
            ) -> #response_body;
        })
    }
}

fn make_operation_spec_name(
    operation_id: Option<&str>,
    operation_type: &str,
    verb: Verb,
    path: &str,
) -> String {
    match operation_id {
        Some(operation_id) => format!("{}{operation_type}", AsUpperCamelCase(operation_id)),
        None => format!(
            "{}{}{operation_type}",
            AsUpperCamelCase(verb.to_string()),
            AsUpperCamelCase(path)
        ),
    }
}

/// Iterate over the OpenApi specification, constructing endpoints anad inserting each into the model.
pub(crate) fn insert_endpoints(spec: &OpenAPI, model: &mut ApiModel<Ref>) -> Result<(), Error> {
    for (path, path_item) in path_items(spec) {
        for (verb, operation) in path_item.iter() {
            let verb: Verb = verb.parse().map_err(|err| Error::UnknownVerb {
                verb: verb.to_string(),
                err,
            })?;
            let endpoint_documentation = path_item
                .description
                .as_deref()
                .or(path_item.summary.as_deref())
                .map(ToOwned::to_owned);
            let operation_documentation = operation
                .description
                .as_deref()
                .or(operation.summary.as_deref())
                .map(ToOwned::to_owned);

            let parameters = {
                // start with the path item parameters
                let path_item_params = path_item.parameters.iter();

                // update with the operation parameters
                let operation_params = operation.parameters.iter();

                // `IndexMap::from_iter` uses the same logic as its `extend`,
                // which lets subsequent items override earlier items.
                path_item_params
                    .chain(operation_params)
                    .map(|param_ref| convert_param_ref(spec, model, param_ref))
                    .collect::<Result<_, _>>()?
            };

            let operation_id = operation.operation_id.clone();

            let request_body = {
                let spec_name =
                    make_operation_spec_name(operation_id.as_deref(), "Request", verb, path);
                operation
                    .request_body
                    .as_ref()
                    .filter(|_request_body| verb.request_body_is_legal())
                    .map(|body_ref| {
                        request_body::create_request_body_from_ref(
                            spec, model, &spec_name, body_ref,
                        )
                    })
                    .transpose()?
            };

            let response = {
                let spec_name =
                    make_operation_spec_name(operation_id.as_deref(), "Response", verb, path);
                response::create_responses(spec, model, &spec_name, &operation.responses)?
            };

            let endpoint = Endpoint::<Ref> {
                path: path.to_string(),
                endpoint_documentation,
                operation_documentation,
                verb,
                parameters,
                operation_id,
                request_body,
                response,
            };

            model.endpoints.push(endpoint);
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown http verb")]
    UnknownVerb {
        verb: String,
        #[source]
        err: strum::ParseError,
    },
    #[error("could not create reference from supplied parameter ref")]
    ConvertParamRef(#[source] anyhow::Error),
    #[error("could not create from supplied request body")]
    CreateRequestBody(#[source] anyhow::Error),
    #[error("could not create from supplied response")]
    CreateResponse(#[source] anyhow::Error),
}
