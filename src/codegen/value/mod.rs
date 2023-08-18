pub(crate) mod list;
pub(crate) mod map;
pub(crate) mod object;
pub(crate) mod one_of_enum;
pub(crate) mod property_override;
pub(crate) mod scalar;
pub(crate) mod set;
pub(crate) mod string_enum;

use crate::codegen::{
    api_model::{self, Ref, Reference, UnknownReference},
    make_ident, ApiModel, List, Map, Object, OneOfEnum, PropertyOverride, Scalar, Set, StringEnum,
};

use openapiv3::{
    ArrayType, IntegerFormat, IntegerType, NumberFormat, NumberType, ObjectType, OpenAPI,
    SchemaData, StringFormat, StringType, VariantOrUnknownOrEmpty,
};
use proc_macro2::TokenStream;
use quote::quote;

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
    PropertyOverride(PropertyOverride<Ref>),
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
            Value::PropertyOverride(property_override) => Ok(Value::PropertyOverride(
                property_override.resolve_refs(resolver)?,
            )),
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
                "mime" | "content-type" => Value::Scalar(Scalar::Mime),
                "accept-header" => Value::Scalar(Scalar::AcceptHeader),
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
        spec: &OpenAPI,
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
                    .convert_reference_or(spec, spec_name, &rust_name, None, items, None)
                    .map_err(ValueConversionError::from_inline(&rust_name))?;
                if array_type.unique_items {
                    Ok(Set::new(item).into())
                } else {
                    Ok(List::new(item).into())
                }
            }
        }
    }

    pub(crate) fn parse_object_type(
        spec: &OpenAPI,
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        object_type: &ObjectType,
    ) -> Result<Self, ValueConversionError> {
        match (
            object_type.properties.is_empty(),
            object_type.additional_properties.as_ref(),
        ) {
            (false, Some(_)) => {
                // can't have both non-empty properties and also additional properties
                Err(ValueConversionError::AdditionalPropertiesConflict)
            }
            (_, Some(additional_properties)) => {
                // string->item mapping
                let map = Map::new(spec, model, spec_name, rust_name, additional_properties)?;
                Ok(map.into())
            }
            (_, None) => {
                // object
                let object = Object::new(spec, model, spec_name, rust_name, object_type)?;
                Ok(object.into())
            }
        }
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
            Value::Scalar(_)
            | Value::List(_)
            | Value::Set(_)
            | Value::Map(_)
            | Value::Ref(_)
            | Value::PropertyOverride(_) => {
                quote!(type)
            }
            Value::StringEnum(_) | Value::OneOfEnum(_) => quote!(enum),
            Value::Object(_) => quote!(struct),
        }
    }

    /// `true` when this is a struct or enum.
    pub fn is_struct_or_enum(&self) -> bool {
        match self {
            Value::Scalar(_)
            | Value::List(_)
            | Value::Set(_)
            | Value::Map(_)
            | Value::Ref(_)
            | Value::PropertyOverride(_) => false,
            Value::StringEnum(_) | Value::OneOfEnum(_) | Value::Object(_) => true,
        }
    }

    pub(crate) fn as_property_override(&self) -> Option<&PropertyOverride<R>> {
        match self {
            Self::PropertyOverride(property_override) => Some(property_override),
            _ => None,
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
            Value::PropertyOverride(property_override) => model
                .resolve(property_override.ref_)
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
            Value::PropertyOverride(property_override) => model
                .resolve(property_override.ref_)
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
            Value::PropertyOverride(property_override) => model
                .resolve(property_override.ref_)
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
            Value::PropertyOverride(property_override) => {
                let name = name_resolver(property_override.ref_)?;
                let ident = make_ident(name);
                Ok(quote!(#ident))
            }
        }
    }

    pub fn serde_container_attributes(&self, is_typedef: bool) -> Vec<TokenStream> {
        let mut out = Vec::new();
        if !is_typedef {
            out.push(quote!(crate = "openapi_gen::reexport::serde"));
        }
        if let Value::OneOfEnum(one_of_enum) = self {
            out.extend(one_of_enum.serde_container_attributes());
        }
        out
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
