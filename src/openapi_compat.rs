use heck::AsUpperCamelCase;
use openapiv3::{
    MediaType, OpenAPI, Operation, Parameter, PathItem, ReferenceOr, Response, Responses, Schema,
};

use crate::{
    codegen::{find_well_known_type, Scalar},
    resolve_trait::Resolve,
};

/// Convert a `StatusCode` enum into a `String` suitable for use as an ident for that code.
pub(crate) fn status_name(code: &openapiv3::StatusCode) -> String {
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

fn is_external<T>(ref_: &ReferenceOr<T>) -> bool {
    ref_.as_ref_str()
        .map(|ref_str| !ref_str.starts_with("#/components/"))
        .unwrap_or_default()
}

fn is_inline<T>(ref_: &ReferenceOr<T>) -> bool {
    ref_.as_item().is_some()
}

fn is_inline_or_external<T>(ref_: &ReferenceOr<T>) -> bool {
    is_inline(ref_) || is_external(ref_)
}

#[derive(Debug, Clone)]
pub(crate) enum OrScalar<T> {
    Item(T),
    Scalar(Scalar),
}

impl<T> From<Scalar> for OrScalar<T> {
    fn from(value: Scalar) -> Self {
        Self::Scalar(value)
    }
}

impl<T> OrScalar<T>
where
    ReferenceOr<T>: Resolve<Output = T>,
{
    /// Convert a `ReferenceOr<T>` into an `OrScalar<T>`.
    ///
    /// Well-known types are converted into the appropriate scalar.
    ///
    /// Unknown references, including those which look like internal references
    /// but are not found,  are converted into `Scalar::Any`.
    pub(crate) fn new<'a>(spec: &'a OpenAPI, ref_: &'a ReferenceOr<T>) -> OrScalar<&'a T> {
        if let Some(wkt) = ref_.as_ref_str().and_then(find_well_known_type) {
            return wkt.into();
        }
        match ref_.resolve(spec) {
            Ok(t) => OrScalar::Item(t),
            Err(_) => OrScalar::Scalar(Scalar::Any),
        }
    }
}

/// Iterate over all `(status, response)` tuples for this `Responses` struct.
///
/// `status` is the canonical status name string, or `"Default"`.
fn all_responses(
    responses: &Responses,
) -> impl '_ + Iterator<Item = (String, &ReferenceOr<Response>)> {
    let status_responses = responses
        .responses
        .iter()
        .map(|(status_code, response_ref)| (status_name(status_code), response_ref));
    let default_response = responses
        .default
        .as_ref()
        .map(|response_ref| ("Default".into(), response_ref));
    status_responses.chain(default_response.into_iter())
}

/// Iterate over all path items for an `OpenAPI` struct.
///
/// This ignores any path items not defined inline in the top-level `paths` construct.
pub(crate) fn path_items(spec: &OpenAPI) -> impl '_ + Iterator<Item = (&str, &PathItem)> {
    spec.paths
        .iter()
        .filter_map(|(path, pathitem_ref)| pathitem_ref.as_item().map(|item| (path.as_str(), item)))
}

/// Iterate over all path operations for an `OpenAPI` struct.
///
/// This returns the tuple `(path, operation_name, operation)`.
pub(crate) fn path_operations(
    spec: &OpenAPI,
) -> impl '_ + Iterator<Item = (&str, &str, &Operation)> {
    path_items(spec).flat_map(|(path, path_item)| {
        path_item
            .iter()
            .map(move |(operation_name, operation)| (path, operation_name, operation))
    })
}

/// Iterate over all request content types defined for this operation.
///
/// Internal references are ok; external references are ignored.
///
/// Items are `(content_type, media_type0)`.
fn operation_request_types<'a>(
    spec: &'a OpenAPI,
    operation: &'a Operation,
) -> impl Iterator<Item = (&'a str, &'a MediaType)> {
    operation
        .request_body
        .as_ref()
        .into_iter()
        .filter_map(|body_ref| body_ref.resolve(spec).ok())
        .flat_map(|request_body| {
            request_body
                .content
                .iter()
                .map(|(content_type, media_type)| (content_type.as_str(), media_type))
        })
}

/// Iterate over all response content types defined for this operation.
///
/// Internal references are ok; external references and references which cannot be resolved are ignored.
///
/// Items are `(status, content_type, media_type)`.
fn operation_response_types<'a>(
    spec: &'a OpenAPI,
    operation: &'a Operation,
) -> impl Iterator<Item = (String, &'a str, &'a MediaType)> {
    all_responses(&operation.responses)
        .filter(|(_status, response_ref)| !is_external(response_ref))
        .filter_map(|(status, response_ref)| {
            let response = Resolve::resolve(response_ref, spec).ok()?;
            Some((status, response))
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

#[derive(Debug, Clone, derive_more::Display)]
pub(crate) enum OperationType {
    #[display(fmt = "requestBody")]
    Request,
    #[display(fmt = "responses.{status}")]
    Response { status: String },
}

#[derive(Debug, Clone, derive_more::Display)]
#[display(fmt = "{path}.{operation_name}.{operation_type}.{content_type}")]
pub(crate) struct SchemaMetadata<'a> {
    pub path: String,
    pub operation_name: String,
    pub content_type: String,
    pub operation_type: OperationType,
    pub schema_or_scalar: OrScalar<&'a Schema>,
}

/// Iterate over all schemas defined for this operation.
///
/// External references are propagated, but internal references are dereferenced.
pub(crate) fn operation_inline_schemas<'operation, 'support>(
    spec: &'operation OpenAPI,
    path: &'support str,
    operation_name: &'support str,
    operation: &'operation Operation,
) -> impl 'support + Iterator<Item = SchemaMetadata<'operation>>
where
    'operation: 'support,
{
    let request_inline_items =
        operation_request_types(spec, operation).filter_map(move |(content_type, media_type)| {
            media_type
                .schema
                .as_ref()
                .filter(|schema_ref| is_inline_or_external(schema_ref))
                .map(|schema_ref| {
                    let schema_or_scalar = OrScalar::new(spec, schema_ref);
                    SchemaMetadata {
                        path: path.to_owned(),
                        operation_name: operation_name.to_owned(),
                        content_type: content_type.to_owned(),
                        operation_type: OperationType::Request,
                        schema_or_scalar,
                    }
                })
        });

    let response_inline_items = operation_response_types(spec, operation).filter_map(
        move |(status, content_type, media_type)| {
            media_type
                .schema
                .as_ref()
                .filter(|schema_ref| is_inline_or_external(schema_ref))
                .map(|schema_ref| {
                    let schema_or_scalar = OrScalar::new(spec, schema_ref);
                    SchemaMetadata {
                        path: path.to_owned(),
                        operation_name: operation_name.to_owned(),
                        content_type: content_type.to_owned(),
                        operation_type: OperationType::Response {
                            status: status.to_owned(),
                        },
                        schema_or_scalar,
                    }
                })
        },
    );

    request_inline_items.chain(response_inline_items)
}

/// Iterate over all parameters defined for an operation.
///
/// Parameters which could not be resolved are ignored. This is expected for external parameters.
pub(crate) fn operation_inline_parameters<'a>(
    spec: &'a OpenAPI,
    operation: &'a Operation,
) -> impl Iterator<Item = &'a Parameter> {
    operation
        .parameters
        .iter()
        .filter_map(|param| param.resolve(spec).ok())
}

/// Iterate over all schemas defined in the `components` section of this spec.
///
/// Items are `(name, schema_ref)`.
pub(crate) fn component_schema_ref(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, &ReferenceOr<Schema>)> {
    spec.components
        .iter()
        .flat_map(|components| components.schemas.iter())
        .map(|(name, schema_ref)| (name.as_str(), schema_ref))
}

/// Iterate over all inline and external schemas defined in the `components` section of this spec.
pub(crate) fn component_inline_and_external_schemas(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, OrScalar<&Schema>)> {
    component_schema_ref(spec)
        .filter(|(_name, schema_ref)| is_inline_or_external(schema_ref))
        .map(|(name, schema_ref)| {
            let schema_or_scalar = OrScalar::new(spec, schema_ref);
            (name, schema_or_scalar)
        })
}

/// Iterate over all parameters defined inline in the `components` section of the spec.
///
/// Items are `(name, parameter)`.
pub(crate) fn component_inline_parameters(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, &Parameter)> {
    spec.components
        .iter()
        .flat_map(|components| components.parameters.iter())
        .filter_map(|(name, param_ref)| param_ref.as_item().map(|param| (name.as_str(), param)))
}
