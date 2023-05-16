use super::Item;
use openapiv3::ReferenceOr;

#[derive(Debug, Clone)]
pub struct Set {
    pub item: Box<ReferenceOr<Item>>,
}
