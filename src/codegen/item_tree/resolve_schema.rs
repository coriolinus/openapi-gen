use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaReference};

use super::{well_known_types::find_well_known_type, Scalar};

/// A schema or a scalar.
#[derive(Debug, Clone, derive_more::From, derive_more::TryInto)]
pub enum SchemaOrScalar<'a> {
    Schema(&'a Schema),
    Scalar(Scalar),
}

/// Resolve a schema reference.
///
/// This is generally compatible with `ReferenceOr<Schema>::resolve`, with the following exceptions:
///
/// - never panics
/// - can resolve certain well-known external types
pub fn resolve_schema<'a>(
    spec: &'a OpenAPI,
    schema_ref: &'a ReferenceOr<Schema>,
) -> Result<SchemaOrScalar<'a>, SchemaResolveError> {
    match schema_ref {
        ReferenceOr::Item(schema) => Ok(schema.into()),
        ReferenceOr::Reference { reference } => {
            let outer = reference;
            let deref = |schema: String| {
                let schema_ref = spec
                    .schemas()
                    .get(&schema)
                    .ok_or(SchemaResolveError::ExternalRef(schema))?;
                match schema_ref {
                    ReferenceOr::Item(schema) => Ok(schema),
                    ReferenceOr::Reference { reference } => {
                        let outer = outer.clone();
                        let inner = reference.clone();
                        Err(SchemaResolveError::NestedReferences { outer, inner })
                    }
                }
            };
            if let Some(scalar) = find_well_known_type(reference) {
                return Ok(scalar.into());
            }
            let reference = SchemaReference::from_str(reference);
            match reference {
                SchemaReference::Schema { schema } => {
                    let schema = deref(schema)?;
                    Ok(schema.into())
                }
                SchemaReference::Property { schema, property } => {
                    let schema = deref(schema)?;
                    let prop = schema
                        .properties()
                        .and_then(|properties| properties.get(&property))
                        .ok_or(SchemaResolveError::UnknownProperty(property))?;
                    resolve_schema(spec, prop)
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchemaResolveError {
    #[error("references outside the current document are generally prohibited ({0})")]
    ExternalRef(String),
    #[error("schema is a reference to schema '{outer}', but that schema is itself a reference to '{inner}'")]
    NestedReferences { outer: String, inner: String },
    #[error("unknown property: '{0}'")]
    UnknownProperty(String),
}
