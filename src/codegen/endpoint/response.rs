use anyhow::{anyhow, Context as _};
use heck::{AsUpperCamelCase, ToSnakeCase, ToUpperCamelCase};
use openapiv3::{Header, OpenAPI, Operation, ReferenceOr, Response, Responses, Schema, StatusCode};

use crate::{
    codegen::{
        api_model::Ref,
        endpoint::{header::create_header, Error},
        find_well_known_type,
        value::{
            object::{ObjectMember, BODY_IDENT},
            one_of_enum,
        },
        Item, Object, OneOfEnum, Reference, Scalar, UnknownReference,
    },
    openapi_compat::is_external,
    resolve_trait::Resolve,
    ApiModel,
};

fn wrap_err<E: Into<anyhow::Error>>(err: E) -> Error {
    Error::CreateResponse(err.into())
}

fn status_code(code: &StatusCode) -> Option<http::StatusCode> {
    match code {
        StatusCode::Code(n) => http::StatusCode::from_u16(*n).ok(),
        StatusCode::Range(_) => None,
    }
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
) -> impl '_ + Iterator<Item = (String, &ReferenceOr<Response>, Option<http::StatusCode>)> {
    let enumerated_responses = responses
        .responses
        .iter()
        .map(|(code, response_ref)| (status_name(code), response_ref, status_code(code)));

    let default_response = responses
        .default
        .iter()
        .map(|response_ref| ("Default".into(), response_ref, None));

    enumerated_responses.chain(default_response)
}

/// Iterate over all combinations of `status_code` and `content_type` valid for this response, combining the names appropriately.
///
/// This means that for status codes with only a single content-type, the produced name is just the human name of the status code.
/// For status codes with multiple content types, they are combined.
fn iter_status_and_content_schemas<'a>(
    base_name: &'a str,
    response: &'a Response,
) -> impl 'a + Iterator<Item = (String, Option<(&'a String, &'a ReferenceOr<Schema>)>)> {
    if response.content.is_empty() {
        Box::new(std::iter::once((base_name.to_owned(), None)))
            as Box<dyn Iterator<Item = (String, Option<(&'a String, &'a ReferenceOr<Schema>)>)>>
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

                    let content_type_and_schema = media_type
                        .schema
                        .as_ref()
                        .map(|schema| (content_type, schema));

                    (spec_name, content_type_and_schema)
                }),
        )
    }
}

/// Every response is composed of some number of response variants.
#[derive(Debug, Clone)]
pub(crate) struct ResponseVariant<Ref = Reference> {
    /// Variant name within the response enum
    spec_name: String,
    /// Reference to the item definition for this response variant.
    ///
    /// Depending on the nature of the response, this might be a bare object for the content type,
    /// or it might be a container object with fields for each of the headers, as well as the response body.
    definition: Ref,
}

impl ResponseVariant<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<ResponseVariant<Reference>, UnknownReference> {
        let ResponseVariant {
            spec_name,
            definition,
        } = self;

        let definition = resolver(&definition)?;

        Ok(ResponseVariant {
            spec_name,
            definition,
        })
    }
}

/// These response variants are the variants associated with a particular resposne.
pub(crate) type ResponseVariants<Ref = Reference> = Vec<ResponseVariant<Ref>>;

/// Make a response object which has fields for each header, and a field for the body.
fn make_response_object_with_headers_and_body<'a>(
    spec: &OpenAPI,
    model: &mut ApiModel<Ref>,
    rust_name: String,
    body: Ref,
    headers: impl Iterator<Item = (&'a String, &'a ReferenceOr<Header>)>,
) -> Result<Ref, Error> {
    let mut object = Object::<Ref> {
        is_generated_body_and_headers: true,
        ..Default::default()
    };

    for (header_name, header_ref) in headers {
        let definition = match header_ref {
            ReferenceOr::Reference { reference } => {
                model.get_named_reference(reference).map_err(wrap_err)?
            }
            ReferenceOr::Item(header) => {
                create_header(spec, model, header_name, None, header).map_err(wrap_err)?
            }
        };

        let mut field_name = header_name.to_snake_case();
        model.deconflict_member_or_variant_ident(&mut field_name);

        object
            .members
            .insert(field_name, ObjectMember::new(definition));
    }

    if object.members.contains_key(BODY_IDENT) {
        return Err(wrap_err(anyhow!("invalid header name: `body`")));
    }

    object
        .members
        .insert(BODY_IDENT.to_owned(), ObjectMember::new(body));

    let ref_ = model
        .add_item(
            Item {
                rust_name,
                value: object.into(),
                ..Default::default()
            },
            None,
        )
        .context("adding computed response item")
        .map_err(wrap_err)?;

    Ok(ref_)
}

/// Create response variants from a `Response`, adding each variant to the model.
///
/// A response variant takes one of two forms:
///
/// - If the variant defines no headers, it is just the body content.
/// - Otherwise, it is a struct with a field for each header, and a field for the content.
///
/// This returns a list of response variants. It does _not_ adjust the model to add them anywhere.
pub(crate) fn create_response_variants(
    spec: &OpenAPI,
    model: &mut ApiModel<Ref>,
    spec_name: &str,
    operation_name: Option<&str>,
    response: &Response,
) -> Result<ResponseVariants<Ref>, Error> {
    let mut variants = Vec::new();

    for (status_name, maybe_content_and_schema) in
        iter_status_and_content_schemas(spec_name, response)
    {
        let mut rust_name = status_name.to_upper_camel_case();
        model.deconflict_member_or_variant_ident(&mut rust_name);

        let (content_type, maybe_schema_ref) = maybe_content_and_schema.unzip();
        let content_type = content_type.map(ToOwned::to_owned);

        let content = match maybe_schema_ref {
            None => {
                // a variant without a schema produces nothing
                model.add_scalar(&status_name, &rust_name, None, Scalar::Unit)
            }
            Some(ref_ @ ReferenceOr::Reference { reference }) if is_external(ref_) => {
                // external references either produce a well-known type, or anything if they're unknown
                let scalar = find_well_known_type(reference).unwrap_or(Scalar::Any);
                model.add_scalar(&status_name, &rust_name, None, scalar)
            }
            // basic references and inline definitions have obvious implementations
            Some(ReferenceOr::Reference { reference }) => model.get_named_reference(reference),
            Some(ReferenceOr::Item(schema)) => model.add_inline_items(
                spec,
                &status_name,
                &rust_name,
                None,
                schema,
                None,
                content_type,
            ),
        }
        .with_context(|| anyhow!("unable to produce variant ref for {rust_name}"))
        .map_err(wrap_err)?;

        // If a response header is defined with the name “Content-Type”, it SHALL be ignored.
        let valid_headers = || {
            response
                .headers
                .iter()
                .filter(|(key, _value)| !key.eq_ignore_ascii_case("content-type"))
        };

        let definition = if valid_headers().count() == 0 {
            content
        } else {
            let mut rust_name = format!(
                "{}{status_name}",
                AsUpperCamelCase(operation_name.unwrap_or_default())
            );
            model.deconflict_ident(&mut rust_name);

            make_response_object_with_headers_and_body(
                spec,
                model,
                rust_name,
                content,
                valid_headers(),
            )?
        };

        variants.push(ResponseVariant {
            spec_name: status_name,
            definition,
        });
    }

    Ok(variants)
}

/// Create a responses enum.
///
/// This will always produce an enum, no matter how many responses are included.
pub(crate) fn create_responses(
    spec: &OpenAPI,
    model: &mut ApiModel<Ref>,
    spec_name: &str,
    responses: &Responses,
) -> Result<Ref, Error> {
    let mut out = OneOfEnum {
        discriminant: Some("status".into()),
        ..Default::default()
    };

    for (status_name, response_ref, maybe_status_code) in named_response_items(responses) {
        // we only need this owned binding in one branch of the following match,
        // but in that case, we need it here for the lifetime
        let variants_owned;
        let variants = match response_ref {
            ReferenceOr::Reference { reference } => {
                // if the response is predefined, then we should already have its variants defined in the spec
                model
                    .response_variants
                    .get(reference)
                    .ok_or_else(|| wrap_err(anyhow!("unable to get variants for {reference}")))?
            }
            ReferenceOr::Item(response) => {
                variants_owned =
                    create_response_variants(spec, model, &status_name, Some(spec_name), response)?;
                &variants_owned
            }
        };

        for ResponseVariant {
            spec_name,
            definition,
        } in variants
        {
            let definition = definition.clone();
            let mapping_name = Some(spec_name.clone());
            let mut variant = one_of_enum::Variant::new(definition, mapping_name);
            variant.status_code = maybe_status_code;
            out.variants.push(variant);
        }
    }

    let rust_name = spec_name.to_upper_camel_case();
    model
        .add_item(
            Item {
                spec_name: spec_name.to_owned(),
                rust_name,
                value: out.into(),
                ..Default::default()
            },
            None,
        )
        .map_err(wrap_err)
}

/// `true` if the operation includes any responses distinguished only by content type
pub(crate) fn has_responses_distinguished_only_by_content_type(
    spec: &OpenAPI,
    operation: &Operation,
) -> bool {
    operation
        .responses
        .responses
        .values()
        .chain(operation.responses.default.iter())
        .filter_map(|response_ref| Resolve::resolve(response_ref, spec).ok())
        .any(|response| response.content.len() > 1)
}
