#![allow(non_camel_case_types)]
pub type IsAwesome = bool;
///arbitrary JSON captured in a `serde_json::Value`
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct ArbitraryJson(openapi_gen::reexport::serde_json::Value);
openapi_gen::newtype_derive_canonical_form!(
    ArbitraryJson, openapi_gen::reexport::serde_json::Value
);
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct Thing {
    pub awesomeness: IsAwesome,
    ///arbitrary JSON captured in a `serde_json::Value`
    pub data: ArbitraryJson,
}
pub type List = Vec<Thing>;
type SetItem = i64;
pub type Set = std::collections::HashSet<SetItem>;
pub type Map = std::collections::HashMap<String, Thing>;
///sort order
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub enum Ordering {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}
type MaybeColor = Option<Color>;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub enum Color {
    #[serde(rename = "red")]
    Red,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "blue")]
    Blue,
}
///discriminated collection types
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "type")]
pub enum Collection {
    List(List),
    Set(Set),
    Map(Map),
}
/**An untagged enum matches the first variant which successfully parses,
so ensure they are distinguishable
*/
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize
)]
#[serde(crate = "openapi_gen::reexport::serde", untagged)]
pub enum UntaggedEnum {
    Thing(Thing),
    Ordering(Ordering),
    MaybeColor(MaybeColor),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {}

