use openapiv3::Schema;

use crate::codegen::{Ref, Reference, UnknownReference};

/// This value type delegates all its type-level stuff to another implementation,
/// but overrides some field-value-relevant information.
#[derive(Debug, Clone)]
pub struct PropertyOverride<Ref = Reference> {
    pub read_only: bool,
    pub write_only: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub ref_: Ref,
}

impl PropertyOverride<Ref> {
    pub fn new(schema: &Schema, ref_: Ref) -> Self {
        Self {
            read_only: schema.schema_data.read_only,
            write_only: schema.schema_data.write_only,
            title: schema.schema_data.title.clone(),
            description: schema.schema_data.description.clone(),
            ref_,
        }
    }

    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<PropertyOverride<Reference>, UnknownReference> {
        let Self {
            read_only,
            write_only,
            title,
            description,
            ref_,
        } = self;

        let ref_ = resolver(&ref_)?;

        Ok(PropertyOverride {
            read_only,
            write_only,
            title,
            description,
            ref_,
        })
    }
}
