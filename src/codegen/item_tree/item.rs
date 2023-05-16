use openapiv3::{Schema, SchemaKind, Type};

use super::{Scalar, Value};

/// Root struct of the abstract item tree used here to model OpenAPI items.
///
/// This ultimately controls everything about how an item is emitted in Rust.
#[derive(Default, Debug, Clone)]
pub struct Item {
    /// Name defined inline. Overrides the generated name.
    pub name: Option<String>,
    /// When true, construct a newtype instead of a typedef.
    pub newtype: bool,
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
            SchemaKind::Type(_type) => match _type {
                Type::Boolean {} => Value::Scalar(Scalar::Bool),
                Type::String(string_type) => string_type.try_into()?,
                Type::Number(number_type) => number_type.try_into()?,
                Type::Integer(integer_type) => integer_type.try_into()?,
                Type::Array(array_type) => array_type.try_into()?,
                Type::Object(object_type) => object_type.try_into()?,
            },
            SchemaKind::OneOf { one_of } => todo!(),
            SchemaKind::AllOf { .. } | SchemaKind::AnyOf { .. } | SchemaKind::Not { .. } => {
                return Err("`allOf`, `anyOf`, and `not` are not supported".into())
            }
        };
        todo!()
    }
}
