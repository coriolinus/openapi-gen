use super::Item;
use indexmap::IndexMap;
use openapiv3::ReferenceOr;

#[derive(Debug, Clone)]
pub struct ObjectMember {
    pub definition: Box<ReferenceOr<Item>>,
    pub required: bool,
    pub read_only: bool,
    pub write_only: bool,
}

/// An inline definition of an object
#[derive(Debug, Clone)]
pub struct Object {
    pub members: IndexMap<String, ObjectMember>,
}
