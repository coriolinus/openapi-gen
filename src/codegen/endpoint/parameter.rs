use anyhow::{anyhow, bail, Context};
use heck::ToUpperCamelCase;
use openapiv3::{OpenAPI, ParameterSchemaOrContent, ReferenceOr};

use crate::{
    codegen::{endpoint::Error, Item, Ref, Reference, UnknownReference, Value},
    resolve_trait::Resolve,
    ApiModel,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, strum::Display, strum::EnumString,
)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
    Cookie,
}

impl<'a> From<&'a openapiv3::Parameter> for ParameterLocation {
    fn from(value: &'a openapiv3::Parameter) -> Self {
        match value {
            openapiv3::Parameter::Query { .. } => ParameterLocation::Query,
            openapiv3::Parameter::Header { .. } => ParameterLocation::Header,
            openapiv3::Parameter::Path { .. } => ParameterLocation::Path,
            openapiv3::Parameter::Cookie { .. } => ParameterLocation::Cookie,
        }
    }
}

/// A unique parameter is defined by a combination of a name and location.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ParameterKey {
    pub name: String,
    pub location: Option<ParameterLocation>,
}

#[derive(Debug, Clone)]
pub struct Parameter<Ref = Reference> {
    pub required: bool,
    pub item_ref: Ref,
}

impl Parameter<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Parameter<Reference>, UnknownReference> {
        let Self { required, item_ref } = self;

        let item_ref = resolver(&item_ref)?;

        Ok(Parameter { required, item_ref })
    }
}

/// Convert a `&Parameter` into a `Ref`
pub(crate) fn insert_parameter(
    spec: &OpenAPI,
    model: &mut ApiModel<Ref>,
    reference_name: Option<&str>,
    param: &openapiv3::Parameter,
) -> anyhow::Result<Ref> {
    let parameter_data = param.parameter_data_ref();
    let spec_name = parameter_data.name.clone();
    let rust_name = spec_name.to_upper_camel_case();
    let schema_ref = match &parameter_data.format {
        ParameterSchemaOrContent::Schema(schema) => schema,
        ParameterSchemaOrContent::Content(content) => {
            // in the parameter context, this map must contain exactly one entry
            if content.len() > 1 {
                bail!("malformed content type: must contain exactly one value");
            }
            content
                .first()
                .and_then(|(_content_type, media_type)| media_type.schema.as_ref())
                .ok_or_else(|| anyhow!("malformed content type: contained no values"))?
        }
    };

    let ref_ = match schema_ref {
        ReferenceOr::Reference { reference } => {
            // if the schema is a reference to something else, we branch our behavior:
            //
            // If the schema is required, we can return the ref directly. Otherwise, we need
            // to create a typedef which wraps it in an Option.
            let inner_ref = model
                .get_named_reference(reference)
                .context("looking up parameter reference")?;
            if parameter_data.required {
                inner_ref
            } else {
                // so let's make a nullable wrapper just pointing to the original item
                let wrapper = Item {
                    docs: parameter_data.description.clone(),
                    spec_name,
                    rust_name,
                    nullable: true,
                    value: Value::Ref(inner_ref),
                    ..Default::default()
                };
                model
                    .add_item(wrapper, reference_name)
                    .context("adding wrapper item for parameter data")?
            }
        }
        ReferenceOr::Item(schema) => {
            // if we defined an inline schema, we need to add the item
            model
                .add_inline_items(spec, &spec_name, &rust_name, reference_name, schema, None)
                .context("adding parameter item")?
        }
    };

    if let Some(named_reference) = reference_name {
        model.insert_named_reference_for(named_reference, &ref_)?;
    }

    Ok(ref_)
}

pub(crate) fn convert_param_ref(
    spec: &OpenAPI,
    model: &mut ApiModel<Ref>,
    param_ref: &ReferenceOr<openapiv3::Parameter>,
) -> Result<(ParameterKey, Parameter<Ref>), Error> {
    // we don't want to be constantly redefining things, so this function has two modes:
    // if the parameter is a reference, then look for that reference among the existing definitions.
    // otherwise, for an inline definition, add it from scratch.
    let item_ref = match param_ref {
        ReferenceOr::Reference { reference } => model
            .get_named_reference(reference)
            .map_err(|err| Error::ConvertParamRef(err.into()))?,

        ReferenceOr::Item(parameter) => {
            insert_parameter(spec, model, None, parameter).map_err(Error::ConvertParamRef)?
        }
    };

    let item = model.resolve(&item_ref).ok_or_else(|| {
        Error::ConvertParamRef(anyhow!(
            "unexpected forward reference converting parameter ref: {param_ref:?}"
        ))
    })?;

    let location = Resolve::resolve(param_ref, spec)
        .ok()
        .map(ParameterLocation::from);

    let parameter_key = ParameterKey {
        name: item.rust_name.clone(),
        location,
    };

    let parameter = Parameter {
        required: !item.nullable,
        item_ref,
    };

    Ok((parameter_key, parameter))
}
