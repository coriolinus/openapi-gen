use heck::AsUpperCamelCase;
use indexmap::IndexMap;
use openapiv3::OpenAPI;

use crate::{openapi_compat::path_items, ApiModel};

use super::api_model::{Ref, Reference, UnknownReference};

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
                    .map(|param_ref| convert_param_ref(model, param_ref))
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
                        request_body::create_request_body_from_ref(model, &spec_name, body_ref)
                    })
                    .transpose()?
            };

            let response = {
                let spec_name =
                    make_operation_spec_name(operation_id.as_deref(), "Response", verb, path);
                response::create_responses(model, &spec_name, &operation.responses)?
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
