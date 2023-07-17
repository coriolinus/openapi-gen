use anyhow::anyhow;
use heck::ToUpperCamelCase;
use openapiv3::{ReferenceOr, Schema};

use crate::{
    codegen::{
        item_tree::{one_of_enum, Item, Value},
        Scalar,
    },
    openapi_compat::is_external,
    ApiModel,
};

use super::{super::api_model::Ref, Error};

fn wrap_err<E: Into<anyhow::Error>>(err: E) -> Error {
    Error::CreateRequestBody(err.into())
}

/// Convert an `Option<ReferenceOr<Schema>>` into an `Item`
///
/// Basic Rules:
///
/// - `None` => `Scalar::Any`
/// - `Some(ReferenceOr::Reference)` => `Value::Ref(_)`
/// - `Some(ReferenceOr::Item(schema))` => Item::parse_schema`
fn convert_optional_schema_ref(
    model: &mut ApiModel<Ref>,
    spec_name: String,
    rust_name: String,
    optional_schema_ref: Option<&ReferenceOr<Schema>>,
) -> Result<Item<Ref>, Error> {
    match optional_schema_ref.as_ref() {
        None => Ok(Item {
            value: Value::Scalar(Scalar::Any),
            spec_name,
            rust_name,
            pub_typedef: true,
            ..Default::default()
        }),
        Some(&ref_ @ ReferenceOr::Reference { reference }) => {
            let value = if is_external(ref_) {
                // external references map to `Any`.
                // todo: warn, in this event.
                Value::from(Scalar::Any)
            } else {
                // internal references just reference the internal definition
                let ref_ = model.get_named_reference(reference).map_err(wrap_err)?;
                Value::Ref(ref_)
            };
            Ok(Item {
                value,
                spec_name,
                rust_name,
                pub_typedef: true,
                ..Default::default()
            })
        }
        Some(ReferenceOr::Item(schema)) => {
            Item::parse_schema(model, &spec_name, &rust_name, schema).map_err(wrap_err)
        }
    }
}

/// Insert an `openapiv3::RequestBody` into the model, producing as `Ref`.
pub(crate) fn create_request_body(
    model: &mut ApiModel<Ref>,
    spec_name: &str,
    reference_name: Option<&str>,
    request_body: &openapiv3::RequestBody,
) -> Result<Ref, Error> {
    let rust_name = spec_name.to_upper_camel_case();
    let mut item = if request_body.content.len() == 1 {
        let optional_schema_ref = request_body
            .content
            .first()
            .and_then(|(_content_type, media_type)| media_type.schema.as_ref());
        convert_optional_schema_ref(model, spec_name.to_owned(), rust_name, optional_schema_ref)?
    } else {
        // someone had the ill grace to produce several different request types differentiated by the `content_type`.
        // this means we can't emit a simple item, but have to turn this into a `OneOf` enum.
        let variants = request_body
            .content
            .iter()
            .map::<Result<_, _>, _>(|(content_type, media_type)| {
                let spec_name = content_type.clone();
                let mut rust_name = spec_name.to_upper_camel_case();
                model.deconflict_member_or_variant_ident(&mut rust_name);

                let variant_item = convert_optional_schema_ref(
                    model,
                    spec_name,
                    rust_name,
                    media_type.schema.as_ref(),
                )?;
                let definition = model.add_item(variant_item, None).map_err(wrap_err)?;
                Ok(one_of_enum::Variant {
                    definition,
                    mapping_name: None,
                })
            })
            .collect::<Result<_, _>>()?;
        let value = one_of_enum::OneOfEnum {
            discriminant: None,
            variants,
        }
        .into();
        Item {
            spec_name: spec_name.to_owned(),
            rust_name,
            value,
            ..Item::default()
        }
    };

    if item.docs.is_none() && request_body.content.len() == 1 {
        item.docs = request_body.description.clone();
    }

    item.nullable = !request_body.required;

    model.add_item(item, reference_name).map_err(wrap_err)
}

/// Convert a `ReferenceOr<openapiv3::RequestBody>` into a `Ref`.
pub(crate) fn create_request_body_from_ref(
    model: &mut ApiModel<Ref>,
    spec_name: &str,
    body_ref: &ReferenceOr<openapiv3::RequestBody>,
) -> Result<Ref, super::Error> {
    match body_ref {
        // reference branch is fairly straightforward: just load the reference
        ReferenceOr::Reference { reference } => Ok(model
            .get_named_reference(reference)
            .map_err(|err| Error::CreateRequestBody(err.into()))?),

        // item branch is a touch more complicated, but not really.
        // we just have to convert the item description, then return the ref.
        ReferenceOr::Item(request_body) => {
            create_request_body(model, spec_name, None, request_body)
        }
    }
}
