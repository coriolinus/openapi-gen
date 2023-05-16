use super::Item;
use openapiv3::ReferenceOr;

/// OpenAPI's string `enum` type
#[derive(Debug, Clone)]
pub struct StringEnum {
    pub variants: Vec<String>,
}

/// OpenAPI's `oneOf` type
#[derive(Debug, Clone)]
pub struct OneOfEnum {
    pub discriminant: Option<String>,
    pub variants: Vec<ReferenceOr<Item>>,
}
