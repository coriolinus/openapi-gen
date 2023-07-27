//! This module is temporary; we want to use https://github.com/kurtbuilds/openapiv3/pull/5 once it is merged.

use anyhow::{anyhow, bail, Result};
use openapiv3::{
    Header, OpenAPI, Parameter, ReferenceOr, RequestBody, Response, Schema, SchemaReference,
};

fn schema_reference_from_str(reference: &str) -> Result<SchemaReference> {
    // limit to 7 items taken here, because that's all we need to know whether a components section
    // matches any of the recognized patterns
    let components = reference.split('/').take(7).collect::<Vec<_>>();
    match components.as_slice() {
        ["#", "components", "schemas", schema] => Ok(SchemaReference::Schema {
            schema: (*schema).to_owned(),
        }),
        ["#", "components", "schemas", schema, "properties", property] => {
            Ok(SchemaReference::Property {
                schema: (*schema).to_owned(),
                property: (*property).to_owned(),
            })
        }
        _ => bail!("malformed reference; {reference} cannot be parsed as SchemaReference"),
    }
}

fn item_or_err<'a, T>(maybe_ref: &'a ReferenceOr<T>, reference: &str) -> Result<&'a T> {
    match maybe_ref {
        ReferenceOr::Item(item) => Ok(item),
        ReferenceOr::Reference { reference: ref_ } => Err(anyhow!(
            "reference {reference} refers to {ref_}"
        )
        .context("references must refer directly to the definition; chains are not permitted")),
    }
}

fn parse_reference<'a>(reference: &'a str, group: &str) -> Result<&'a str> {
    reference
        .rsplit_once('/')
        .filter(|(head, _name)| head.strip_prefix("#/components/") == Some(group))
        .map(|(_head, name)| name)
        .ok_or_else(|| anyhow!("invalid {} reference: {}", group, reference))
}

/// Abstract over types which can potentially resolve a contained type, given an `OpenAPI` instance.
pub trait Resolve {
    type Output;

    fn resolve<'a>(&'a self, spec: &'a OpenAPI) -> Result<&'a Self::Output>;
}

impl<'o> Resolve for &'o ReferenceOr<Schema> {
    type Output = Schema;

    fn resolve<'a>(&'a self, spec: &'a OpenAPI) -> Result<&'a Self::Output> {
        let reference = match self {
            ReferenceOr::Item(item) => return Ok(item),
            ReferenceOr::Reference { reference } => reference,
        };
        let schema_ref = schema_reference_from_str(reference)?;
        let get_schema = |schema: &str| -> Result<&Schema> {
            let schema_ref = spec
                .schemas()
                .get(schema)
                .ok_or_else(|| anyhow!("{reference} not found in OpenAPI spec"))?;
            item_or_err(schema_ref, reference)
        };
        match &schema_ref {
            SchemaReference::Schema { schema } => get_schema(schema),
            SchemaReference::Property {
                schema: schema_name,
                property,
            } => {
                let schema = get_schema(schema_name)?;
                let schema_ref = schema
                    .properties()
                    .ok_or_else(|| anyhow!("tried to resolve reference {reference}, but {schema_name} is not an object with properties"))?
                    .get(property).ok_or_else(|| anyhow!("schema {schema_name} lacks property {property}"))?;
                Resolve::resolve(schema_ref, spec)
            }
        }
    }
}

impl Resolve for ReferenceOr<Schema> {
    type Output = Schema;

    fn resolve<'a>(&'a self, spec: &'a OpenAPI) -> Result<&'a Self::Output> {
        Resolve::resolve(self, spec)
    }
}

macro_rules! impl_resolve_for {
    (ReferenceOr<$output:ident>; $getter:ident; $components_field:ident) => {
        impl<'o> Resolve for &'o ReferenceOr<$output> {
            type Output = $output;

            fn resolve<'a>(&'a self, spec: &'a OpenAPI) -> Result<&'a Self::Output> {
                match self {
                    ReferenceOr::Item(item) => Ok(item),
                    ReferenceOr::Reference { reference } => {
                        let name = $getter(reference)?;
                        let components = spec
                            .components
                            .as_ref()
                            .ok_or_else(|| anyhow!("no components in spec"))?;
                        let param_ref = components
                            .$components_field
                            .get(name)
                            .ok_or_else(|| anyhow!("{reference} not found in OpenAPI spec"))?;
                        item_or_err(param_ref, reference)
                    }
                }
            }
        }

        impl Resolve for ReferenceOr<$output> {
            type Output = $output;

            fn resolve<'a>(&'a self, spec: &'a OpenAPI) -> Result<&'a Self::Output> {
                Resolve::resolve(self, spec)
            }
        }
    };
}

fn get_response_name(reference: &str) -> Result<&str> {
    parse_reference(reference, "responses")
}
impl_resolve_for!(ReferenceOr<Response>; get_response_name; responses);

fn get_parameter_name(reference: &str) -> Result<&str> {
    parse_reference(reference, "parameters")
}
impl_resolve_for!(ReferenceOr<Parameter>; get_parameter_name; parameters);

fn get_request_body_name(reference: &str) -> Result<&str> {
    parse_reference(reference, "requestBodies")
}
impl_resolve_for!(ReferenceOr<RequestBody>; get_request_body_name; request_bodies);

fn get_header_name(reference: &str) -> Result<&str> {
    parse_reference(reference, "headers")
}
impl_resolve_for!(ReferenceOr<Header>; get_header_name; headers);
