use openapiv3::{Schema, SchemaKind, Type};

use super::{OneOfEnum, Scalar, Value};

fn get_extension_value<'a>(schema: &'a Schema, key: &str) -> Option<&'a serde_json::Value> {
    schema.schema_data.extensions.get(key)
}

fn get_extension_bool(schema: &Schema, key: &str) -> bool {
    get_extension_value(schema, key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or_default()
}

/// Root struct of the abstract item tree used here to model OpenAPI items.
///
/// This ultimately controls everything about how an item is emitted in Rust.
#[derive(Default, Debug, Clone)]
pub struct Item {
    /// Documentation to be injected for this item.
    pub docs: Option<String>,
    /// Name defined inline. Overrides the generated name.
    pub name: Option<String>,
    /// When true, construct a newtype instead of a typedef.
    pub newtype: bool,
    /// When true and `newtype` is set, derive [`derive_more::From`].
    pub newtype_from: bool,
    /// When true and `newtype` is set, derive [`derive_more::Into`].
    pub newtype_into: bool,
    /// When true and `newtype` is set, derive [`derive_more::Deref`].
    pub newtype_deref: bool,
    /// When true and `newtype` is set, derive [`derive_more::DerefMut`].
    pub newtype_deref_mut: bool,
    /// When true, emits an outer typedef around an `Option`, and an inner item definition.
    pub nullable: bool,
    /// What value this item contains
    pub value: Value,
}

impl From<Value> for Item {
    fn from(value: Value) -> Self {
        Item {
            value,
            ..Default::default()
        }
    }
}

/// Code generation is going to be a whole lot easier given an AST that maps to Rust types, rather than one which
/// conforms to OpenAPI semantics. So, let's build such an AST.
impl<'a> TryFrom<&'a Schema> for Item {
    type Error = String;

    fn try_from(schema: &'a Schema) -> Result<Self, Self::Error> {
        let value = match &schema.schema_kind {
            SchemaKind::Any(_) => Scalar::Any.into(),
            SchemaKind::Type(Type::Boolean {}) => Value::Scalar(Scalar::Bool),
            SchemaKind::Type(Type::String(string_type)) => string_type.try_into()?,
            SchemaKind::Type(Type::Number(number_type)) => number_type.try_into()?,
            SchemaKind::Type(Type::Integer(integer_type)) => integer_type.try_into()?,
            SchemaKind::Type(Type::Array(array_type)) => array_type.try_into()?,
            SchemaKind::Type(Type::Object(object_type)) => object_type.try_into()?,
            SchemaKind::OneOf { one_of } => {
                OneOfEnum::try_from(&schema.schema_data, one_of)?.into()
            }
            SchemaKind::AllOf { .. } | SchemaKind::AnyOf { .. } | SchemaKind::Not { .. } => {
                return Err("`allOf`, `anyOf`, and `not` are not supported".into())
            }
        };

        // Get documentation from the provided external documentation, or alternately from the description.
        let docs = schema
            .schema_data
            .external_docs
            .as_ref()
            .map(|docs| reqwest::blocking::get(&docs.url))
            .transpose()
            .map_err(|err| err.to_string())?
            .map(|response| response.text())
            .transpose()
            .map_err(|err| err.to_string())?
            .or_else(|| schema.schema_data.description.clone());

        let name = get_extension_value(schema, "name")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);

        let newtype = get_extension_bool(schema, "newtype");
        let newtype_from = get_extension_bool(schema, "newtypeFrom");
        let newtype_into = get_extension_bool(schema, "newtypeInto");
        let newtype_deref = get_extension_bool(schema, "newtypeDeref");
        let newtype_deref_mut = get_extension_bool(schema, "newtypeDerefMut");

        let nullable = schema.schema_data.nullable;

        Ok(Self {
            docs,
            name,
            newtype,
            newtype_from,
            newtype_into,
            newtype_deref,
            newtype_deref_mut,
            nullable,
            value,
        })
    }
}
