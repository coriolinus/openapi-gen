use super::{maybe_map_reference_or, Item};
use openapiv3::{ReferenceOr, Schema, SchemaData};

/// OpenAPI's string `enum` type
#[derive(Debug, Clone)]
pub struct StringEnum {
    pub variants: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub definition: ReferenceOr<Item>,
    pub mapping_name: Option<String>,
}

/// OpenAPI's `oneOf` type
#[derive(Debug, Clone)]
pub struct OneOfEnum {
    pub discriminant: Option<String>,
    pub variants: Vec<Variant>,
}

impl OneOfEnum {
    pub fn try_from(
        schema_data: &SchemaData,
        variants: &[ReferenceOr<Schema>],
    ) -> Result<Self, String> {
        let discriminant = schema_data
            .discriminator
            .as_ref()
            .map(|discriminant| discriminant.property_name.clone());

        let variants = variants
            .iter()
            .map(|schema_ref| {
                let definition =
                    maybe_map_reference_or(schema_ref.as_ref(), |schema| schema.try_into())?;

                let mapping_name = schema_data
                    .discriminator
                    .as_ref()
                    .and_then(|discriminator| {
                        discriminator.mapping.iter().find_map(|(name, reference)| {
                            (Some(reference.as_str()) == schema_ref.as_ref_str())
                                .then(|| name.to_owned())
                        })
                    });

                Ok(Variant {
                    definition,
                    mapping_name,
                })
            })
            .collect::<Result<_, String>>()?;

        Ok(Self {
            discriminant,
            variants,
        })
    }
}
