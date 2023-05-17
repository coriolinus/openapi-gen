use openapiv3::{
    ArrayType, IntegerFormat, IntegerType, NumberFormat, NumberType, ObjectType, ReferenceOr,
    StringFormat, StringType, VariantOrUnknownOrEmpty,
};

use super::{
    maybe_map_reference_or, List, Map, Object, ObjectMember, OneOfEnum, Scalar, Set, StringEnum,
};

/// The fundamental value type.
///
/// This type doesn't capture all of the information needed for codegen, but it
/// is the heart of the type abstraction.
#[derive(Debug, Clone, derive_more::From)]
pub enum Value {
    Scalar(Scalar),
    List(List),
    Set(Set),
    StringEnum(StringEnum),
    OneOfEnum(OneOfEnum),
    Object(Object),
    Map(Map),
}

impl Default for Value {
    fn default() -> Self {
        Self::Scalar(Scalar::Any)
    }
}

impl TryFrom<&StringType> for Value {
    type Error = String;

    fn try_from(string_type: &StringType) -> Result<Self, Self::Error> {
        // TODO: handle `string_type.pattern`

        if !matches!(&string_type.format, VariantOrUnknownOrEmpty::Empty)
            && !string_type.enumeration.is_empty()
        {
            return Err("cannot specify both format and enum".into());
        }
        let value = match &string_type.format {
            VariantOrUnknownOrEmpty::Item(format) => match format {
                StringFormat::Binary => Value::Scalar(Scalar::Binary),
                #[cfg(feature = "bytes")]
                StringFormat::Byte => Value::Scalar(Scalar::Bytes),
                #[cfg(not(feature = "bytes"))]
                StringFormat::Byte => Value::Scalar(Scalar::String),
                StringFormat::Date => Value::Scalar(Scalar::Date),
                StringFormat::DateTime => Value::Scalar(Scalar::DateTime),
                StringFormat::Password => Value::Scalar(Scalar::String),
            },
            VariantOrUnknownOrEmpty::Unknown(format) => match format.to_lowercase().as_str() {
                #[cfg(feature = "bytes")]
                "base64" => Value::Scalar(Scalar::Bytes),
                "ip" => Value::Scalar(Scalar::IpAddr),
                "ipv4" => Value::Scalar(Scalar::Ipv4Addr),
                "ipv6" => Value::Scalar(Scalar::Ipv6Addr),
                #[cfg(feature = "uuid")]
                "uuid" => Value::Scalar(Scalar::Uuid),
                // unknown string types are valid and devolve to `String`
                _ => Value::Scalar(Scalar::String),
            },
            VariantOrUnknownOrEmpty::Empty => {
                if string_type.enumeration.is_empty() {
                    Value::Scalar(Scalar::String)
                } else {
                    Value::StringEnum(StringEnum {
                        variants: string_type.enumeration.clone(),
                    })
                }
            }
        };
        Ok(value)
    }
}

impl TryFrom<&NumberType> for Value {
    type Error = String;

    fn try_from(number_type: &NumberType) -> Result<Self, Self::Error> {
        let value = match &number_type.format {
            VariantOrUnknownOrEmpty::Item(format) => match format {
                NumberFormat::Float => Value::Scalar(Scalar::F32),
                NumberFormat::Double => Value::Scalar(Scalar::F64),
            },
            VariantOrUnknownOrEmpty::Empty => Value::Scalar(Scalar::F64),
            VariantOrUnknownOrEmpty::Unknown(unk) => {
                return Err(format!("unknown number format: {unk}"))
            }
        };
        Ok(value)
    }
}

impl TryFrom<&IntegerType> for Value {
    type Error = String;

    fn try_from(integer_type: &IntegerType) -> Result<Self, Self::Error> {
        macro_rules! by_format {
            ($t32:expr, $t64:expr) => {
                match &integer_type.format {
                    VariantOrUnknownOrEmpty::Item(IntegerFormat::Int32) => Value::Scalar($t32),
                    VariantOrUnknownOrEmpty::Item(IntegerFormat::Int64) => Value::Scalar($t64),
                    VariantOrUnknownOrEmpty::Empty => Value::Scalar($t64),
                    VariantOrUnknownOrEmpty::Unknown(unk) => {
                        return Err(format!("unknown integer format: {unk}"))
                    }
                }
            };
        }

        // TODO: handle `integer_type.multiple_of`
        let value = {
            #[cfg(feature = "integer-restrictions")]
            {
                by_format!(
                    Scalar::i32_from(integer_type),
                    Scalar::i64_from(integer_type)
                )
            }
            #[cfg(not(feature = "integer-restrictions"))]
            {
                by_format!(Scalar::I32, Scalar::I64)
            }
        };
        Ok(value)
    }
}

impl TryFrom<&ArrayType> for Value {
    type Error = String;

    fn try_from(array_type: &ArrayType) -> Result<Self, Self::Error> {
        let item = match &array_type.items {
            None => Box::new(ReferenceOr::item(Value::Scalar(Scalar::Any).into())),
            Some(items_ref) => {
                let items_ref = items_ref.as_ref();
                Box::new(maybe_map_reference_or(items_ref, |schema| {
                    (&**schema).try_into()
                })?)
            }
        };
        if array_type.unique_items {
            Ok(Set { item }.into())
        } else {
            Ok(List { item }.into())
        }
    }
}

impl TryFrom<&ObjectType> for Value {
    type Error = String;

    fn try_from(object_type: &ObjectType) -> Result<Self, Self::Error> {
        match (
            !object_type.properties.is_empty(),
            object_type.additional_properties.is_some(),
        ) {
            (true, true) => Err(
                "object cannot define non-empty `properties` and also `additionalProperties`"
                    .into(),
            ),
            (_, true) => {
                // mapping
                let additional_properties = object_type.additional_properties.as_ref().unwrap();
                let value_type = match additional_properties {
                    openapiv3::AdditionalProperties::Any(_) => None,
                    openapiv3::AdditionalProperties::Schema(schema_ref) => {
                        let schema_ref = schema_ref.as_ref().as_ref();
                        Some(Box::new(maybe_map_reference_or(schema_ref, |schema| {
                            schema.try_into()
                        })?))
                    }
                };
                Ok(Value::Map(Map { value_type }))
            }
            _ => {
                // object
                let members = object_type
                    .properties
                    .iter()
                    .map::<Result<_, String>, _>(|(name, schema_ref)| {
                        let required = object_type.required.contains(name);

                        // TODO: should we resolve references here instead? But that kind of doesn't make sense.
                        // For now, we're only permitting inline definitions to be read/write-only.
                        let read_only = schema_ref
                            .as_item()
                            .map(|schema| schema.schema_data.read_only)
                            .unwrap_or_default();

                        let write_only = schema_ref
                            .as_item()
                            .map(|schema| schema.schema_data.write_only)
                            .unwrap_or_default();

                        let definition =
                            Box::new(maybe_map_reference_or(schema_ref.as_ref(), |schema| {
                                schema.try_into()
                            })?);
                        Ok((
                            name.to_owned(),
                            ObjectMember {
                                required,
                                definition,
                                read_only,
                                write_only,
                            },
                        ))
                    })
                    .collect::<Result<_, _>>()?;
                Ok(Value::Object(Object { members }))
            }
        }
    }
}
