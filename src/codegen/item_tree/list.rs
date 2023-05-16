use super::Item;
use openapiv3::ReferenceOr;

#[derive(Debug, Clone)]
pub struct List {
    pub item: Box<ReferenceOr<Item>>,
}
