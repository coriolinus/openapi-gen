use std::borrow::Cow;

use openapiv3::{
    ArrayType, IntegerFormat, IntegerType, NumberFormat, NumberType, ObjectType, ReferenceOr,
    SchemaData, StringFormat, StringType, VariantOrUnknownOrEmpty,
};
use proc_macro2::TokenStream;
use quote::quote;

use super::{
    maybe_map_reference_or, Item, List, Map, Object, ObjectMember, OneOfEnum, Scalar, Set,
    StringEnum,
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

impl TryFrom<&NumberType> for Value {
    type Error = String;

    fn try_from(number_type: &NumberType) -> Result<Self, Self::Error> {
        let value = match &number_type.format {
            VariantOrUnknownOrEmpty::Item(NumberFormat::Float) => Value::Scalar(Scalar::F32),
            VariantOrUnknownOrEmpty::Item(NumberFormat::Double)
            | VariantOrUnknownOrEmpty::Empty => Value::Scalar(Scalar::F64),
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
        // TODO: handle `integer_type.multiple_of`
        let value = match &integer_type.format {
            VariantOrUnknownOrEmpty::Item(IntegerFormat::Int32) => {
                Value::Scalar(Scalar::integer_32_from(integer_type))
            }
            VariantOrUnknownOrEmpty::Item(IntegerFormat::Int64)
            | VariantOrUnknownOrEmpty::Empty => {
                Value::Scalar(Scalar::integer_64_from(integer_type))
            }
            VariantOrUnknownOrEmpty::Unknown(unk) => {
                return Err(format!("unknown integer format: {unk}"))
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

impl Value {
    pub fn try_from_string_type(
        string_type: &StringType,
        schema_data: &SchemaData,
    ) -> Result<Self, String> {
        // TODO: handle `string_type.pattern`

        let x_extensible_enum = schema_data
            .extensions
            .get("x-extensible-enum")
            .and_then(|value| value.as_array())
            .map(|values| {
                values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .map(ToOwned::to_owned)
                            .ok_or("x-extensible-enum item must be a string".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let enumeration = match (
            string_type.enumeration.is_empty(),
            x_extensible_enum.is_empty(),
        ) {
            (false, false) => {
                return Err("cannot specify both `enum` and `x-extensible-enum`".into())
            }
            (true, false) => x_extensible_enum,
            (false, true) => string_type.enumeration.clone(),
            (true, true) => Vec::new(),
        };

        if !matches!(&string_type.format, VariantOrUnknownOrEmpty::Empty) && !enumeration.is_empty()
        {
            return Err("cannot specify both format and enumeration".into());
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
                if enumeration.is_empty() {
                    Value::Scalar(Scalar::String)
                } else {
                    Value::StringEnum(StringEnum {
                        variants: enumeration
                            .into_iter()
                            .filter(|e| !schema_data.nullable || e != "null")
                            .collect(),
                    })
                }
            }
        };
        Ok(value)
    }

    /// Iterate over sub-items, not includeing `self`.
    ///
    /// This only includes inline definitions. It makes no attempt to resolve references.
    pub fn sub_items<'a>(&'a self) -> Box<dyn 'a + Iterator<Item = (Cow<'a, str>, &'a Item)>> {
        match self {
            Value::Scalar(_) | Value::StringEnum(_) => Box::new(std::iter::empty()),
            Value::List(list) => Box::new(
                list.item
                    .as_item()
                    .map(|item| (Cow::from("Item"), item))
                    .into_iter(),
            ),
            Value::Set(set) => Box::new(
                set.item
                    .as_item()
                    .map(|item| (Cow::from("Item"), item))
                    .into_iter(),
            ),
            Value::OneOfEnum(one_of) => Box::new(one_of.variants.iter().enumerate().filter_map(
                |(idx, variant)| {
                    variant
                        .definition
                        .as_item()
                        .map(|item| (format!("Variant{idx}").into(), item))
                },
            )),
            Value::Object(object) => {
                Box::new(object.members.iter().filter_map(|(name, member)| {
                    member
                        .definition
                        .as_item()
                        .map(|item| (name.as_str().into(), item))
                }))
            }
            Value::Map(map) => Box::new(
                map.value_type
                    .as_ref()
                    .and_then(|item_ref| item_ref.as_item())
                    .map(|item| (Cow::from("Item"), item))
                    .into_iter(),
            ),
        }
    }

    /// What kind of item keyword does this value type use?
    pub fn item_keyword(&self) -> TokenStream {
        match self {
            Value::Scalar(_) | Value::List(_) | Value::Set(_) | Value::Map(_) => quote!(type),
            Value::StringEnum(_) | Value::OneOfEnum(_) => quote!(enum),
            Value::Object(_) => quote!(struct),
        }
    }

    /// `true` when this is a struct or enum.
    pub fn is_struct_or_enum(&self) -> bool {
        match self {
            Value::Scalar(_) | Value::List(_) | Value::Set(_) | Value::Map(_) => false,
            Value::StringEnum(_) | Value::OneOfEnum(_) | Value::Object(_) => true,
        }
    }

    /// Emit the bare form of the item definition.
    ///
    /// This omits visibility, identifier, and any miscellaneous punctuation (`=`; `;`).
    ///
    /// This includes necessary punctuation such as `{` and `}` surrounding a struct definition.
    pub fn emit_item_definition(&self, derived_name: &str) -> TokenStream {
        match self {
            Value::Scalar(scalar) => scalar.emit_type(),
            Value::List(list) => list.emit_definition(derived_name),
            Value::Set(set) => set.emit_definition(derived_name),
            Value::Map(map) => map.emit_definition(derived_name),
            Value::StringEnum(string_enum) => string_enum.emit_definition(derived_name),
            Value::OneOfEnum(one_of_enum) => one_of_enum.emit_definition(derived_name),
            Value::Object(object) => object.emit_definition(derived_name),
        }
    }
}
