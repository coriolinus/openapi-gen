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
pub struct ArbitraryJson(openapi_gen::reexport::serde_json::Value);
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    openapi_gen::reexport::derive_more::Constructor
)]
pub struct Thing {
    pub awesomeness: IsAwesome,
    ///arbitrary JSON captured in a `serde_json::Value`
    pub data: ArbitraryJson,
}
type List = Vec<Thing>;
type SetItem = i64;
type Set = std::collections::HashSet<SetItem>;
type Map = std::collections::HashMap<String, Thing>;
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
pub enum Ordering {
    Asc,
    Desc,
}
type MaybeColor = Option<Color>;
pub enum Color {
    Red,
    Green,
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
#[serde(tag = "type")]
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
#[serde(untagged)]
pub enum UntaggedEnum {
    Thing(Thing),
    Ordering(Ordering),
    MaybeColor(MaybeColor),
}
