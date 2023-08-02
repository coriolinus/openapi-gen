use heck::ToUpperCamelCase;
use openapiv3::{OpenAPI, ParameterSchemaOrContent, ReferenceOr};

use crate::{
    codegen::{api_model, Item, Ref, Scalar, Value},
    ApiModel,
};

/// Insert an `openapiv3::Header` into the model, producing as `Ref`.
pub(crate) fn create_header(
    spec: &OpenAPI,
    model: &mut ApiModel<Ref>,
    spec_name: &str,
    reference_name: Option<&str>,
    header: &openapiv3::Header,
) -> Result<Ref, Error> {
    let mut rust_name = spec_name.to_upper_camel_case();
    model.deconflict_ident(&mut rust_name);
    let model_err = |context: &'static str| {
        move |err| Error::ModifyModel(spec_name.to_owned(), context.to_owned(), Box::new(err))
    };

    let maybe_schema_ref = match &header.format {
        ParameterSchemaOrContent::Schema(schema) => Some(schema),
        ParameterSchemaOrContent::Content(content) => {
            // in the header context, this map must contain exactly one entry
            if content.len() > 1 {
                return Err(Error::TooManyContentTypes(spec_name.to_owned()));
            }
            content
                .first()
                .and_then(|(_content_type, media_type)| media_type.schema.as_ref())
        }
    };

    let ref_ = match maybe_schema_ref {
        None => model
            .add_scalar(spec_name, &rust_name, reference_name, Scalar::Any)
            .map_err(model_err("adding `Any` scalar for unspecified schema"))?,
        Some(ReferenceOr::Reference { reference }) => {
            // if the schema is a reference to something else, we branch our behavior:
            //
            // If the schema is required, we can return the ref directly. Otherwise, we need
            // to create a typedef which wraps it in an Option.
            let inner_ref = model
                .get_named_reference(reference)
                .map_err(model_err("looking up parameter reference"))?;
            if header.required {
                inner_ref
            } else {
                // so let's make a nullable wrapper just pointing to the original item
                let wrapper = Item {
                    docs: header.description.clone(),
                    spec_name: spec_name.to_owned(),
                    rust_name,
                    nullable: true,
                    value: Value::Ref(inner_ref),
                    ..Default::default()
                };
                model
                    .add_item(wrapper, reference_name)
                    .map_err(model_err("adding wrapper item for parameter data"))?
            }
        }
        Some(ReferenceOr::Item(schema)) => {
            // if we defined an inline schema, we need to add the item
            model
                .add_inline_items(spec, spec_name, &rust_name, reference_name, schema, None)
                .map_err(model_err("adding parameter item"))?
        }
    };

    if let Some(named_reference) = reference_name {
        model
            .insert_named_reference_for(named_reference, &ref_)
            .map_err(model_err("inserting named reference"))?;
    }

    Ok(ref_)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("header ({0}): content type must contain at most one value")]
    TooManyContentTypes(String),
    #[error("header ({0}): {1}")]
    ModifyModel(String, String, #[source] Box<api_model::Error>),
}
