use heck::AsUpperCamelCase;
use openapiv3::{
    MediaType, OpenAPI, Operation, Parameter, PathItem, ReferenceOr, Response, Responses, Schema,
};

/// Convert a `StatusCode` enum into a `String` suitable for use as an ident for that code.
pub fn status_name(code: &openapiv3::StatusCode) -> String {
    match code {
        openapiv3::StatusCode::Code(n) => http::StatusCode::from_u16(*n)
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

/// Iterate over all `(status, response)` tuples for this `Responses` struct.
///
/// `status` is the canonical status name string, or `"Default"`.
fn all_responses(
    responses: &Responses,
) -> impl '_ + Iterator<Item = (String, ReferenceOr<&Response>)> {
    let status_responses = responses
        .responses
        .iter()
        .map(|(status_code, response_ref)| (status_name(status_code), response_ref.as_ref()));
    let default_response = responses
        .default
        .as_ref()
        .map(|response_ref| ("Default".into(), response_ref.as_ref()));
    status_responses.chain(default_response.into_iter())
}

/// Iterate over all path items for an `OpenAPI` struct.
///
/// This ignores any path items not defined inline in the top-level `paths` construct.
pub fn path_items(spec: &OpenAPI) -> impl '_ + Iterator<Item = (&str, &PathItem)> {
    spec.paths
        .iter()
        .filter_map(|(path, pathitem_ref)| pathitem_ref.as_item().map(|item| (path.as_str(), item)))
}

/// Iterate over all path operations for an `OpenAPI` struct.
///
/// This returns the tuple `(path, operation_name, operation)`.
pub fn path_operations(spec: &OpenAPI) -> impl '_ + Iterator<Item = (&str, &str, &Operation)> {
    path_items(spec).flat_map(|(path, path_item)| {
        path_item
            .iter()
            .map(move |(operation_name, operation)| (path, operation_name, operation))
    })
}

/// Iterate over all inline request content types defined for this operation.
///
/// Items are `(content_type, media_type0)`.
fn operation_inline_request_types(
    operation: &Operation,
) -> impl Iterator<Item = (&str, &MediaType)> {
    operation
        .request_body
        .as_ref()
        .into_iter()
        .filter_map(|body_ref| body_ref.as_item())
        .flat_map(|request_body| {
            request_body
                .content
                .iter()
                .map(|(content_type, media_type)| (content_type.as_str(), media_type))
        })
}

/// Iterate over all inline response content types defined for this operation.
///
/// Items are `(status, content_type, media_type)`.
fn operation_inline_response_types(
    operation: &Operation,
) -> impl Iterator<Item = (String, &str, &MediaType)> {
    all_responses(&operation.responses)
        .filter_map(|(status, response_ref)| {
            response_ref
                .as_item()
                .map(move |response| (status, *response))
        })
        .flat_map(|(status, response)| {
            response
                .content
                .iter()
                .map(move |(content_type, media_type)| {
                    (status.clone(), content_type.as_str(), media_type)
                })
        })
}

/// Iterate over all inline items defined for this operation.
///
/// Items are `(derived_name, schema)`.
pub fn operation_inline_schemas<'operation, 'support>(
    path: &'support str,
    operation_name: &'support str,
    operation: &'operation Operation,
) -> impl 'support + Iterator<Item = (String, &'operation Schema)>
where
    'operation: 'support,
{
    let request_inline_items =
        operation_inline_request_types(operation).filter_map(move |(content_type, media_type)| {
            media_type.schema.as_ref().and_then(|schema_ref| {
                schema_ref.as_item().map(|schema| {
                    let derived_name = format!(
                        "{}{}Request{}",
                        AsUpperCamelCase(path),
                        AsUpperCamelCase(operation_name),
                        AsUpperCamelCase(content_type)
                    );
                    (derived_name, schema)
                })
            })
        });

    let response_inline_items = operation_inline_response_types(operation).filter_map(
        move |(status, content_type, media_type)| {
            media_type.schema.as_ref().and_then(|schema_ref| {
                schema_ref.as_item().map(|schema| {
                    let derived_name = format!(
                        "{}{}{}Response{}",
                        AsUpperCamelCase(path),
                        AsUpperCamelCase(operation_name),
                        AsUpperCamelCase(status),
                        AsUpperCamelCase(content_type)
                    );
                    (derived_name, schema)
                })
            })
        },
    );

    request_inline_items.chain(response_inline_items)
}

/// Iterate over all parameters defined for an operation.
///
/// Parameters which could not be resolved are ignored.
pub fn operation_inline_parameters(operation: &Operation) -> impl Iterator<Item = &Parameter> {
    operation
        .parameters
        .iter()
        .filter_map(|param| param.as_item())
}

/// Iterate over all schemas defined in the `components` section of this spec.
///
/// Items are `(name, schema_ref)`.
pub fn component_schema_ref(spec: &OpenAPI) -> impl Iterator<Item = (&str, &ReferenceOr<Schema>)> {
    spec.components
        .iter()
        .flat_map(|components| components.schemas.iter())
        .map(|(name, schema_ref)| (name.as_str(), schema_ref))
}

/// Iterate over all schemas defined inline in the `components` section of this spec.
///
/// Items are `(name, schema)`.
///
/// This does not account for schemas which are type references to other schemas.
pub fn component_schemas(spec: &OpenAPI) -> impl Iterator<Item = (&str, &Schema)> {
    component_schema_ref(spec)
        .filter_map(|(name, schema_ref)| schema_ref.as_item().map(|schema| (name, schema)))
}

/// Iterate over all parameters defined inline in the `components` section of the spec.
///
/// Items are `(name, parameter)`.
pub fn component_parameters(spec: &OpenAPI) -> impl Iterator<Item = (&str, &Parameter)> {
    spec.components
        .iter()
        .flat_map(|components| components.parameters.iter())
        .filter_map(|(name, param_ref)| {
            param_ref
                .resolve(spec)
                .ok()
                .map(|param| (name.as_str(), param))
        })
}
