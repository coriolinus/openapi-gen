use super::Item;
use openapiv3::ReferenceOr;

/// An inline definition of a mapping from String to T
#[derive(Debug, Clone)]
pub struct Map {
    pub value_type: Option<Box<ReferenceOr<Item>>>,
}
