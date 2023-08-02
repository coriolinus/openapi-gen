use openapiv3::{Header, OpenAPI, Parameter, PathItem, ReferenceOr, RequestBody, Response, Schema};

use crate::resolve_trait::Resolve;

pub(crate) mod or_scalar;
pub(crate) use or_scalar::OrScalar;

pub(crate) fn is_external<T>(ref_: &ReferenceOr<T>) -> bool {
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

/// Iterate over all path items for an `OpenAPI` struct.
///
/// This ignores any path items not defined inline in the top-level `paths` construct.
/// (Path items are not a standardized member of the components.)
pub(crate) fn path_items(spec: &OpenAPI) -> impl '_ + Iterator<Item = (&str, &PathItem)> {
    spec.paths
        .iter()
        .filter_map(|(path, pathitem_ref)| pathitem_ref.as_item().map(|item| (path.as_str(), item)))
}

/// Iterate over all schemas defined in the `components` section of this spec.
///
/// Items are `(schema_name, reference_name, schema_ref)`.
fn component_schema_ref(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, String, &ReferenceOr<Schema>)> {
    spec.components
        .iter()
        .flat_map(|components| components.schemas.iter())
        .map(|(name, schema_ref)| {
            (
                name.as_str(),
                format!("#/components/schemas/{name}"),
                schema_ref,
            )
        })
}

/// Iterate over all inline and external schemas defined in the `components` section of this spec.
///
/// Items are `(spec_name, reference_name, schema_or_scalar)`.
pub(crate) fn component_inline_and_external_schemas(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, String, OrScalar<&Schema>)> {
    component_schema_ref(spec)
        .filter(|(_name, _ref_name, schema_ref)| is_inline_or_external(schema_ref))
        .map(|(name, reference_name, schema_ref)| {
            let schema_or_scalar = OrScalar::new(spec, schema_ref);
            (name, reference_name, schema_or_scalar)
        })
}

/// Iterate over all parameters defined in the `components` section of the spec.
///
/// Items are `(spec_name, reference_name, parameter)`.
pub(crate) fn component_parameters(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, String, &Parameter)> {
    spec.components
        .iter()
        .flat_map(|components| components.parameters.iter())
        .filter_map(|(name, param_ref)| {
            Resolve::resolve(param_ref, spec).ok().map(|param| {
                (
                    name.as_str(),
                    format!("#/components/parameters/{name}"),
                    param,
                )
            })
        })
}

/// Iterate over all named requests from the `components` section of the spec.
///
/// Items are `(spec_name, reference_name, request_body)`
pub(crate) fn component_requests(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, String, &RequestBody)> {
    spec.components
        .iter()
        .flat_map(|components| components.request_bodies.iter())
        .filter_map(|(name, request_ref)| {
            Resolve::resolve(request_ref, spec).ok().map(|request| {
                (
                    name.as_str(),
                    format!("#/components/requestBodies/{name}"),
                    request,
                )
            })
        })
}

/// Iterate over all named responses from the `components` section of the spec.
///
/// Items are `(spec_name, reference_name, response)`.
pub(crate) fn component_responses(
    spec: &OpenAPI,
) -> impl Iterator<Item = (&str, String, &Response)> {
    spec.components
        .iter()
        .flat_map(|components| components.responses.iter())
        .filter_map(|(name, response_ref)| {
            Resolve::resolve(response_ref, spec).ok().map(|response| {
                (
                    name.as_str(),
                    format!("#/components/responses/{name}"),
                    response,
                )
            })
        })
}

/// Iterate over all named headers from the `components` section of the spec.
///
/// Items are `(spec_name, reference_name, header)`.
pub(crate) fn component_headers(spec: &OpenAPI) -> impl Iterator<Item = (&str, String, &Header)> {
    spec.components
        .iter()
        .flat_map(|components| components.headers.iter())
        .filter_map(|(name, header_ref)| {
            Resolve::resolve(header_ref, spec).ok().map(|header| {
                (
                    name.as_str(),
                    format!("#/components/headers/{name}"),
                    header,
                )
            })
        })
}
