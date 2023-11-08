use anyhow::{anyhow, bail, Context};
use heck::ToUpperCamelCase;
use openapiv3::{OpenAPI, ParameterSchemaOrContent, ReferenceOr};

use crate::{
    codegen::{endpoint::Error, Ref, Reference, UnknownReference},
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

#[derive(Debug, Clone)]
pub struct Parameter<Ref = Reference> {
    pub rust_name: String,
    pub spec_name: String,
    pub location: ParameterLocation,
    pub required: bool,
    pub item_ref: Ref,
}

impl Parameter<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Parameter<Reference>, UnknownReference> {
        let Self {
            rust_name,
            spec_name,
            location,
            required,
            item_ref,
        } = self;

        let item_ref = resolver(&item_ref)?;

        Ok(Parameter {
            rust_name,
            spec_name,
            location,
            required,
            item_ref,
        })
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
    let mut content_type = None;
    let schema_ref = match &parameter_data.format {
        ParameterSchemaOrContent::Schema(schema) => schema,
        ParameterSchemaOrContent::Content(content) => {
            // in the parameter context, this map must contain exactly one entry
            if content.len() > 1 {
                bail!("malformed content type: must contain exactly one value");
            }
            content
                .first()
                .and_then(|(content_type_str, media_type)| {
                    content_type = Some(content_type_str.to_owned());
                    media_type.schema.as_ref()
                })
                .ok_or_else(|| anyhow!("malformed content type: contained no values"))?
        }
    };

    let ref_ = match schema_ref {
        ReferenceOr::Reference { reference } => {
            // conveniently, this never needs to be optional:
            //
            // - path parameters must always be required
            // - query parameters are collected into a parameter object, which has its own avenue for optionality
            // - header parameters are not collected into a single object but have special-casing for optionality
            // - cookie parameters are not supported
            //
            // as such, we can always just return the reference
            model
                .get_named_reference(reference)
                .context("looking up parameter reference")?
        }
        ReferenceOr::Item(schema) => {
            // if we defined an inline schema, we need to add the item
            model
                .add_inline_items(
                    spec,
                    &spec_name,
                    &rust_name,
                    reference_name,
                    schema,
                    None,
                    content_type,
                )
                .context("adding parameter item")?
        }
    };

    if let Some(named_reference) = reference_name {
        model.insert_named_reference_for(named_reference, &ref_)?;
    }

    // best-effort ensure that we `impl Header` wherever necessary
    if matches!(param, openapiv3::Parameter::Header { .. }) {
        if let Ok(item) = model.resolve_mut(&ref_) {
            item.impl_header = true;
        }
    }

    Ok(ref_)
}

pub(crate) fn convert_param_ref(
    spec: &OpenAPI,
    model: &mut ApiModel<Ref>,
    param_ref: &ReferenceOr<openapiv3::Parameter>,
) -> Result<Parameter<Ref>, Error> {
    // we get the parameter the simple way this time, as a shorthand for determining certain parameters
    let param = Resolve::resolve(param_ref, spec).map_err(|err| {
        Error::ConvertParamRef(anyhow!(err).context("failed to resolve parameter reference"))
    })?;
    let required = param.parameter_data_ref().required;
    let spec_name = param.parameter_data_ref().name.clone();
    let location = ParameterLocation::from(param);

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

    let item = model.resolve(&item_ref).map_err(|err| {
        Error::ConvertParamRef(
            anyhow!(err)
                .context("unexpected forward reference converting parameter ref: {param_ref:?}"),
        )
    })?;

    let rust_name = item.rust_name.clone();

    let parameter = Parameter {
        rust_name,
        spec_name,
        location,
        required,
        item_ref,
    };

    Ok(parameter)
}
