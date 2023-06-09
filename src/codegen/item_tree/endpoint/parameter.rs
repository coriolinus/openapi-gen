use anyhow::anyhow;
use heck::ToUpperCamelCase;
use openapiv3::{ParameterSchemaOrContent, ReferenceOr};

use crate::ApiModel;

use super::{
    super::api_model::{Ref, Reference, UnknownReference},
    Error,
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

pub(crate) fn convert_param_ref(
    model: &mut ApiModel<Ref>,
    param_ref: &ReferenceOr<openapiv3::Parameter>,
) -> Result<(ParameterKey, Parameter<Ref>), Error> {
    // we don't want to be constantly redefining things, so this function has two modes:
    // if the parameter is a reference, then look for that reference among the existing definitions.
    // otherwise, for an inline definition, add it from scratch.
    let item_ref = match param_ref {
        ReferenceOr::Reference { reference } => {
            model.get_named_reference(reference).ok_or_else(|| {
                Error::ConvertParamRef(anyhow!("named reference '{reference}' not found"))
            })?
        }

        ReferenceOr::Item(parameter) => {
            let parameter_data = parameter.parameter_data_ref();
            let spec_name = parameter_data.name.clone();
            let rust_name = spec_name.to_upper_camel_case();
            let schema_ref = match &parameter_data.format {
                ParameterSchemaOrContent::Schema(schema) => schema,
                ParameterSchemaOrContent::Content(content) => {
                    // in the parameter context, this map must contain exactly one entry
                    if content.len() > 1 {
                        return Err(Error::ConvertParamRef(anyhow!(
                            "malformed content type: must contain exactly one value"
                        )));
                    }
                    content
                        .first()
                        .and_then(|(_content_type, media_type)| media_type.schema.as_ref())
                        .ok_or_else(|| {
                            Error::ConvertParamRef(anyhow!(
                                "malformed content type: contained no values"
                            ))
                        })?
                }
            };
            model
                .convert_reference_or(&spec_name, &rust_name, &schema_ref.as_ref())
                .map_err(|err| Error::ConvertParamRef(err.into()))?
        }
    };
    let item = model.resolve_mut(&item_ref).ok_or_else(|| {
        Error::ConvertParamRef(anyhow!(
            "unexpected forward reference converting parameter ref: {param_ref:?}"
        ))
    })?;

    // we want to backfill some data here for which we need to refer back to the model
    if item.docs.is_none() {
        if let Some(description) = param_ref
            .as_item()
            .map(|param| param.parameter_data_ref().description.clone())
        {
            item.docs = description;
        }
    }

    let location = param_ref.as_item().map(ParameterLocation::from);
    let required = param_ref
        .as_item()
        .map(|parameter| parameter.parameter_data_ref().required)
        .unwrap_or_default();

    let parameter_key = ParameterKey {
        name: item.rust_name.clone(),
        location,
    };

    let parameter = Parameter { required, item_ref };

    Ok((parameter_key, parameter))
}
