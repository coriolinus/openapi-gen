use anyhow::{anyhow, Context as _};
use heck::ToUpperCamelCase;
use openapiv3::{ReferenceOr, Response, Responses, Schema, StatusCode};

use crate::{
    codegen::{api_model::Ref, find_well_known_type, one_of_enum, Item, OneOfEnum, Scalar, Value},
    openapi_compat::is_external,
    ApiModel,
};

use super::Error;

fn wrap_err<E: Into<anyhow::Error>>(err: E) -> Error {
    Error::CreateResponse(err.into())
}

/// Convert a `StatusCode` enum into a `String` describing that status.
fn status_name(code: &StatusCode) -> String {
    match code {
        StatusCode::Code(n) => http::StatusCode::from_u16(*n)
            .ok()
            .and_then(|status| status.canonical_reason())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("status code {n}")),
        StatusCode::Range(r) => match r {
            1 => "informational range".into(),
            2 => "success range".into(),
            3 => "redirection range".into(),
            4 => "client error range".into(),
            5 => "server error range".into(),
            _ => format!("status range {r}xx"),
        },
    }
}

fn named_response_items(
    responses: &Responses,
) -> impl '_ + Iterator<Item = (String, &ReferenceOr<Response>)> {
    let enumerated_responses = responses
        .responses
        .iter()
        .map(|(code, response_ref)| (status_name(code), response_ref));

    let default_response = responses
        .default
        .iter()
        .map(|response_ref| ("Default".into(), response_ref));

    enumerated_responses.chain(default_response)
}

/// Iterate over all combinations of `status_code` and `content_type` valid for this response, combining the names appropriately.
///
/// This means that for status codes with only a single content-type, the produced name is just the human name of the status code.
/// For status codes with multiple content types, they are combined.
fn iter_status_and_content_schemas<'a>(
    base_name: &'a str,
    response: &'a Response,
) -> impl 'a + Iterator<Item = (String, Option<&'a ReferenceOr<Schema>>)> {
    if response.content.is_empty() {
        Box::new(std::iter::once((base_name.to_owned(), None)))
            as Box<dyn Iterator<Item = (String, Option<&'a ReferenceOr<Schema>>)>>
    } else {
        let append_suffix = response.content.len() != 1;
        Box::new(
            response
                .content
                .iter()
                .map(move |(content_type, media_type)| {
                    let spec_name = if append_suffix {
                        format!("{base_name} {content_type}")
                    } else {
                        base_name.to_owned()
                    };

                    let schema = media_type.schema.as_ref();

                    (spec_name, schema)
                }),
        )
    }
}

/// Create a response variant.
///
/// This function is called from two distinct code paths:
///
/// - populating `#/components/responses/*`
/// - creating a variant of a `Responses` enum inline
///
/// This influences its return type: in the first case, the set of responses is
/// fixed, but in the second, they might be part of a larger enumeration across
/// various statuses.
///
/// At the same time, we don't want to create an iterator: that iterator would
/// own mutable access to `model` for its entire lifetime, which is suboptimal.
///
/// So we require a return parameter which is able to accept various `Ref`s.
pub(crate) fn create_response_variants(
    model: &mut ApiModel<Ref>,
    spec_name: &str,
    reference_name: Option<&str>,
    response: &Response,
    refs: &mut impl Extend<(String, Ref)>,
) -> Result<(), Error> {
    for (spec_name, maybe_schema_ref) in iter_status_and_content_schemas(spec_name, response) {
        let rust_name = spec_name.to_upper_camel_case();
        let definition = match maybe_schema_ref {
            None => {
                // a variant without a schema produces nothing
                model.add_scalar(&spec_name, &rust_name, reference_name, Scalar::Unit)
            }
            Some(ref_ @ ReferenceOr::Reference { reference }) if is_external(ref_) => {
                // external references either produce a well-known type, or anything if they're unknown
                let scalar = find_well_known_type(reference).unwrap_or(Scalar::Any);
                model.add_scalar(&spec_name, &rust_name, reference_name, scalar)
            }
            // basic references and inline definitions have obvious implementations
            Some(ReferenceOr::Reference { reference }) => model.get_named_reference(reference),
            Some(ReferenceOr::Item(schema)) => {
                model.add_inline_items(&spec_name, &rust_name, reference_name, schema)
            }
        }
        .with_context(|| anyhow!("unable to produce variant ref for {rust_name}"))
        .map_err(wrap_err)?;
        refs.extend(std::iter::once((rust_name, definition)));
    }
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub(crate) struct ResponseCollector {
    one_of_enum: OneOfEnum<Ref>,
    definition_buffer: Vec<(String, Ref)>,
}

impl Extend<(String, Ref)> for ResponseCollector {
    fn extend<T: IntoIterator<Item = (String, Ref)>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        let min_items = iter.size_hint().0;
        self.definition_buffer.clear();
        self.definition_buffer.reserve(min_items);

        self.definition_buffer.extend(iter);

        self.one_of_enum
            .variants
            .reserve(self.definition_buffer.len());
        self.one_of_enum
            .variants
            .extend(self.definition_buffer.drain(..).map(|(name, definition)| {
                one_of_enum::Variant {
                    definition,
                    mapping_name: Some(name),
                }
            }));
    }
}

impl From<ResponseCollector> for OneOfEnum<Ref> {
    fn from(value: ResponseCollector) -> Self {
        value.one_of_enum
    }
}

impl From<ResponseCollector> for Value<Ref> {
    fn from(value: ResponseCollector) -> Self {
        value.one_of_enum.into()
    }
}

impl ResponseCollector {
    /// convert this `ResponseCollector` into an `Item` and add it to the model, returning a `Ref`.
    pub(crate) fn add_as_item(
        self,
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        reference_name: Option<&str>,
    ) -> Result<Ref, Error> {
        let item = Item {
            value: self.into(),
            spec_name: spec_name.to_owned(),
            rust_name: rust_name.to_owned(),
            ..Default::default()
        };
        model.add_item(item, reference_name).map_err(wrap_err)
    }
}

/// Create a responses enum.
///
/// This will always produce an enum, no matter how many responses are included.
pub(crate) fn create_responses(
    model: &mut ApiModel<Ref>,
    spec_name: &str,
    responses: &Responses,
) -> Result<Ref, Error> {
    let mut response_collector = ResponseCollector::default();

    for (status_code, response_ref) in named_response_items(responses) {
        let mut rust_name = spec_name.to_upper_camel_case();
        model.deconflict_member_or_variant_ident(&mut rust_name);

        let response = match response_ref {
            ReferenceOr::Item(response) => response,
            ReferenceOr::Reference { reference } => {
                let ref_ = model.get_named_reference(reference).map_err(wrap_err)?;
                return Ok(ref_);
            }
        };

        create_response_variants(model, &status_code, None, response, &mut response_collector)?;
    }

    let rust_name = spec_name.to_upper_camel_case();
    response_collector.add_as_item(model, spec_name, &rust_name, None)
}
