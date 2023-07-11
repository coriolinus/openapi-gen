use heck::AsUpperCamelCase;
use openapiv3::{
    AnySchema, ArrayType, IntegerFormat, IntegerType, NumberFormat, NumberType, ObjectType,
    ReferenceOr, SchemaData, StringFormat, StringType, VariantOrUnknownOrEmpty,
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::make_ident;

use super::{
    api_model::{self, Ref, Reference, UnknownReference},
    one_of_enum::Variant,
    ApiModel, List, Map, Object, ObjectMember, OneOfEnum, Scalar, Set, StringEnum,
};

/// The fundamental value type.
///
/// This type doesn't capture all of the information needed for codegen, but it
/// is the heart of the type abstraction.
#[derive(Debug, Clone, derive_more::From)]
pub enum Value<Ref = Reference> {
    Scalar(Scalar),
    StringEnum(StringEnum),
    OneOfEnum(OneOfEnum<Ref>),
    Set(Set<Ref>),
    List(List<Ref>),
    Object(Object<Ref>),
    Map(Map<Ref>),
    #[from(ignore)]
    Ref(Ref),
}

impl<R> Default for Value<R> {
    fn default() -> Self {
        Self::Scalar(Scalar::Any)
    }
}

impl Value<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Value<Reference>, UnknownReference> {
        match self {
            Value::Scalar(scalar) => Ok(Value::Scalar(scalar)),
            Value::StringEnum(string_enum) => Ok(Value::StringEnum(string_enum)),
            Value::OneOfEnum(one_of_enum) => {
                Ok(Value::OneOfEnum(one_of_enum.resolve_refs(resolver)?))
            }
            Value::Set(set) => Ok(Value::Set(set.resolve_refs(resolver)?)),
            Value::List(list) => Ok(Value::List(list.resolve_refs(resolver)?)),
            Value::Object(object) => Ok(Value::Object(object.resolve_refs(resolver)?)),
            Value::Map(map) => Ok(Value::Map(map.resolve_refs(resolver)?)),
            Value::Ref(ref_) => Ok(Value::Ref(resolver(&ref_)?)),
        }
    }

    pub(crate) fn parse_string_type(
        string_type: &StringType,
        schema_data: &SchemaData,
    ) -> Result<Self, ValueConversionError> {
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
                            .ok_or(ValueConversionError::ExtensibleEnumInvalidItem)
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let (enumeration, extensible) = match (
            string_type.enumeration.is_empty(),
            x_extensible_enum.is_empty(),
        ) {
            (false, false) => return Err(ValueConversionError::EnumConflict),
            (true, false) => (x_extensible_enum, true),
            (false, true) => (string_type.enumeration.clone(), false),
            (true, true) => (Vec::new(), false),
        };

        if !matches!(&string_type.format, VariantOrUnknownOrEmpty::Empty) && !enumeration.is_empty()
        {
            return Err(ValueConversionError::FormatEnumConflict);
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
                        extensible,
                    })
                }
            }
        };
        Ok(value)
    }

    pub(crate) fn parse_array_type(
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        array_type: &ArrayType,
    ) -> Result<Self, ValueConversionError> {
        match &array_type.items {
            None => Ok(Self::default()),
            Some(items) => {
                let rust_name = format!("{rust_name}Item");
                let item = model
                    .convert_reference_or(spec_name, &rust_name, items)
                    .map_err(ValueConversionError::from_inline(&rust_name))?;
                if array_type.unique_items {
                    Ok(Set { item }.into())
                } else {
                    Ok(List { item }.into())
                }
            }
        }
    }

    pub(crate) fn parse_object_type(
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        object_type: &ObjectType,
    ) -> Result<Self, ValueConversionError> {
        if !object_type.properties.is_empty() && object_type.additional_properties.is_some() {
            return Err(ValueConversionError::AdditionalPropertiesConflict);
        }

        // string->item mapping
        if let Some(additional_properties) = object_type.additional_properties.as_ref() {
            let value_type = match additional_properties {
                openapiv3::AdditionalProperties::Any(_) => None,
                openapiv3::AdditionalProperties::Schema(schema_ref) => {
                    let rust_name = format!("{rust_name}Item");
                    let item = model
                        .convert_reference_or(spec_name, &rust_name, &schema_ref.as_ref().as_ref())
                        .map_err(ValueConversionError::from_inline(&rust_name))?;
                    Some(item)
                }
            };
            return Ok(Value::Map(Map { value_type }));
        }

        // object
        let members = object_type
            .properties
            .iter()
            .map::<Result<_, ValueConversionError>, _>(|(member_name, schema_ref)| {
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

                // If a model exists for the bare property name, qualify
                // this one with the object name.
                //
                // This isn't perfect, because the first member we
                // encounter with a particular name gets to keep the
                // bare name, while others get their names qualified,
                // but it's still better than just appending digits.
                let ucc_member_name = format!("{}", AsUpperCamelCase(member_name));
                let mut rust_name = if model.ident_exists(&ucc_member_name) {
                    format!("{rust_name}{ucc_member_name}")
                } else {
                    ucc_member_name
                };
                model.deconflict_ident(&mut rust_name);

                let definition = model
                    .convert_reference_or(member_name, &rust_name, &schema_ref.as_ref())
                    .map_err(ValueConversionError::from_inline(&rust_name))?;

                // In a perfect world, we could just set the item's `nullable` field here in the
                // event that we've determined that the item is in fact nullable.
                // Unfortunately, there are two obstacles to this.
                //
                // The first is relatively minor: the definition here might be a forward reference,
                // in which case we don't know that it is nullable in all contexts. We can work around that.
                //
                // The second is a bigger problem, though: making an item nullable the "proper" way
                // involves creating a type alias, which changes its public reference name. This doesn't
                // invalidate past references to the item--the structure of `ApiModel` avoids that issue--
                // but it does mean we'd need to design a single-purpose function `ApiModel::make_item_nullable`,
                // which just feels kind of ugly.
                //
                // Instead, we'll just handle nullable fields inline; that's good enough.
                let inline_option = !object_type.required.contains(member_name);

                Ok((
                    member_name.to_owned(),
                    ObjectMember {
                        definition,
                        read_only,
                        write_only,
                        inline_option,
                    },
                ))
            })
            .collect::<Result<_, _>>()?;
        Ok(Value::Object(Object { members }))
    }

    pub(crate) fn parse_one_of_type(
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        schema_data: &SchemaData,
        variants: &[ReferenceOr<openapiv3::Schema>],
    ) -> Result<Self, ValueConversionError> {
        let discriminant = schema_data
            .discriminator
            .as_ref()
            .map(|discriminant| discriminant.property_name.clone());

        let variants = variants
            .iter()
            .map(|schema_ref| {
                let mapping_name = schema_data
                    .discriminator
                    .as_ref()
                    .and_then(|discriminator| {
                        discriminator.mapping.iter().find_map(|(name, reference)| {
                            // it might seem incomplete to just pull out the `ref` variant of the
                            // `schema_ref` here, but that's actually per the docs:
                            //
                            // <https://docs.rs/openapiv3-extended/latest/openapiv3/struct.Discriminator.html>
                            //
                            // > When using the discriminator, inline schemas will not be considered.
                            (schema_ref.as_ref_str() == Some(reference.as_str()))
                                .then(|| name.to_owned())
                        })
                    });

                let definition = model
                    .convert_reference_or(spec_name, rust_name, &schema_ref.as_ref())
                    .map_err(ValueConversionError::from_inline(rust_name))?;

                Ok(Variant {
                    definition,
                    mapping_name,
                })
            })
            .collect::<Result<_, _>>()?;

        Ok(Value::OneOfEnum(OneOfEnum {
            discriminant,
            variants,
        }))
    }

    /// OpenAPI has a somewhat silly rule: to define a nullable enum, you must explicitly include `null` among the
    /// stated enum variants. The OpenAPI schema model we're using doesn't handle that case well; see
    /// <https://github.com/kurtbuilds/openapiv3/issues/3>.
    ///
    /// This function tries to work around that problem.
    pub(crate) fn try_parse_string_enum_type(
        schema_data: &SchemaData,
        any_schema: &AnySchema,
    ) -> Option<Self> {
        use serde_json::Value;

        // This function is implemented as a giant `Option` combinator.
        //
        // It's formed of two branches, each of which is of the form
        // `condition.then(|| make_a_some_variant())`.

        (schema_data.nullable
            && any_schema.typ.as_deref() == Some("string")
            && any_schema
                .enumeration
                .iter()
                .any(|enum_item| matches!(enum_item, Value::Null))
            && any_schema.enumeration.iter().all(|enum_item| {
                matches!(enum_item, Value::Null) || matches!(enum_item, Value::String(_))
            }))
        .then(|| {
            Self::StringEnum(StringEnum {
                variants: any_schema
                    .enumeration
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToOwned::to_owned)
                    .collect(),
                extensible: false,
            })
        })
        .or_else(|| {
            const X_EXTENSIBLE_ENUM: &str = "x-extensible-enum";
            (schema_data.nullable
                && any_schema.typ.as_deref() == Some("string")
                && schema_data
                    .extensions
                    .get(X_EXTENSIBLE_ENUM)
                    .and_then(Value::as_array)
                    .map(|array| {
                        array.iter().any(|value| matches!(value, Value::Null))
                            && array.iter().all(|value| {
                                matches!(value, Value::Null) || matches!(value, Value::String(_))
                            })
                    })
                    .unwrap_or_default())
            .then(|| {
                Self::StringEnum(StringEnum {
                    variants: schema_data
                        .extensions
                        .get(X_EXTENSIBLE_ENUM)
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(ToOwned::to_owned)
                        .collect(),
                    extensible: true,
                })
            })
        })
    }
}

impl<R> TryFrom<&NumberType> for Value<R> {
    type Error = ValueConversionError;

    fn try_from(number_type: &NumberType) -> Result<Self, Self::Error> {
        let value = match &number_type.format {
            VariantOrUnknownOrEmpty::Item(NumberFormat::Float) => Value::Scalar(Scalar::F32),
            VariantOrUnknownOrEmpty::Item(NumberFormat::Double)
            | VariantOrUnknownOrEmpty::Empty => Value::Scalar(Scalar::F64),
            VariantOrUnknownOrEmpty::Unknown(format) => {
                let format = format.clone();
                return Err(ValueConversionError::UnknownFormat {
                    type_: "number".into(),
                    format,
                });
            }
        };
        Ok(value)
    }
}

impl<R> TryFrom<&IntegerType> for Value<R> {
    type Error = ValueConversionError;

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
            VariantOrUnknownOrEmpty::Unknown(format) => {
                let format = format.clone();
                return Err(ValueConversionError::UnknownFormat {
                    type_: "integer".into(),
                    format,
                });
            }
        };
        Ok(value)
    }
}

impl<R> Value<R> {
    /// What kind of item keyword does this value type use?
    pub fn item_keyword(&self) -> TokenStream {
        match self {
            Value::Scalar(_) | Value::List(_) | Value::Set(_) | Value::Map(_) | Value::Ref(_) => {
                quote!(type)
            }
            Value::StringEnum(_) | Value::OneOfEnum(_) => quote!(enum),
            Value::Object(_) => quote!(struct),
        }
    }

    /// `true` when this is a struct or enum.
    pub fn is_struct_or_enum(&self) -> bool {
        match self {
            Value::Scalar(_) | Value::List(_) | Value::Set(_) | Value::Map(_) | Value::Ref(_) => {
                false
            }
            Value::StringEnum(_) | Value::OneOfEnum(_) | Value::Object(_) => true,
        }
    }
}

impl Value {
    pub fn impls_eq(&self, model: &ApiModel) -> bool {
        match self {
            Value::StringEnum(_) => true,
            Value::Scalar(scalar) => scalar.impls_eq(),
            Value::List(list) => model[list.item].value.impls_eq(model),
            Value::Set(set) => model[set.item].value.impls_eq(model),
            Value::Map(map) => map
                .value_type
                .map(|item| model[item].value.impls_eq(model))
                .unwrap_or(true),
            Value::OneOfEnum(oo_enum) => oo_enum
                .variants
                .iter()
                .all(|variant| model[variant.definition].value.impls_eq(model)),
            Value::Object(object) => object
                .members
                .values()
                .all(|member| model[member.definition].value.impls_eq(model)),
            Value::Ref(ref_) => model
                .resolve(*ref_)
                .map(|item| item.value.impls_eq(model))
                .unwrap_or_default(),
        }
    }

    pub fn impls_copy(&self, model: &ApiModel) -> bool {
        match self {
            Value::List(_) | Value::Map(_) | Value::Set(_) => false,
            Value::StringEnum(string_enum) => string_enum.impls_copy(),
            Value::Scalar(scalar) => scalar.impls_copy(),
            Value::OneOfEnum(oo_enum) => oo_enum
                .variants
                .iter()
                .all(|variant| model[variant.definition].value.impls_copy(model)),
            Value::Object(object) => object
                .members
                .values()
                .all(|member| model[member.definition].value.impls_copy(model)),
            Value::Ref(ref_) => model
                .resolve(*ref_)
                .map(|item| item.value.impls_copy(model))
                .unwrap_or_default(),
        }
    }

    pub fn impls_hash(&self, model: &ApiModel) -> bool {
        match self {
            Value::Map(_) | Value::Set(_) => false,
            Value::StringEnum(_) => true,
            Value::Scalar(scalar) => scalar.impls_hash(),
            Value::List(list) => model[list.item].value.impls_hash(model),
            Value::OneOfEnum(oo_enum) => oo_enum
                .variants
                .iter()
                .all(|variant| model[variant.definition].value.impls_hash(model)),
            Value::Object(object) => object
                .members
                .values()
                .all(|member| model[member.definition].value.impls_hash(model)),
            Value::Ref(ref_) => model
                .resolve(*ref_)
                .map(|item| item.value.impls_hash(model))
                .unwrap_or_default(),
        }
    }

    /// Emit the bare form of the item definition.
    ///
    /// This omits visibility, identifier, and any miscellaneous punctuation (`=`; `;`).
    ///
    /// This includes necessary punctuation such as `{` and `}` surrounding a struct definition.
    pub fn emit_item_definition<'a>(
        &self,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        match self {
            Value::Scalar(scalar) => Ok(scalar.emit_type()),
            Value::StringEnum(string_enum) => Ok(string_enum.emit_definition()),
            Value::List(list) => list.emit_definition(name_resolver),
            Value::Set(set) => set.emit_definition(name_resolver),
            Value::Map(map) => map.emit_definition(name_resolver),
            Value::OneOfEnum(one_of_enum) => one_of_enum.emit_definition(name_resolver),
            Value::Object(object) => object.emit_definition(model, name_resolver),
            Value::Ref(ref_) => {
                let name = name_resolver(*ref_)?;
                let ident = make_ident(name);
                Ok(quote!(#ident))
            }
        }
    }

    pub fn serde_container_attributes(&self) -> Vec<TokenStream> {
        match self {
            Value::OneOfEnum(one_of_enum) => one_of_enum.serde_container_attributes(),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValueConversionError {
    #[error("unknown {type_} format: {format}")]
    UnknownFormat { type_: String, format: String },
    #[error("x-extensible-enum item must be a string")]
    ExtensibleEnumInvalidItem,
    #[error("cannot specify both `enum` and `x-extensible-enum`")]
    EnumConflict,
    #[error("cannot specify both format and enumeration")]
    FormatEnumConflict,
    #[error("computing inline item definition for '{name}'")]
    ComputingInlineItem {
        name: String,
        #[source]
        source: Box<api_model::Error>,
    },
    #[error("object cannot define non-empty `properties` and also `additionalProperties`")]
    AdditionalPropertiesConflict,
}

impl ValueConversionError {
    fn from_inline(name: &str) -> impl '_ + Fn(api_model::Error) -> ValueConversionError {
        move |err| ValueConversionError::ComputingInlineItem {
            name: name.to_owned(),
            source: Box::new(err),
        }
    }
}
