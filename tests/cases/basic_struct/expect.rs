///unsigned integer
type Foo = u64;
type Bar = String;
///this object is defined separately, intended to be used within a reference
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
pub struct InnerStruct {
    ///unsigned integer
    pub foo: Foo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar: Option<Bar>,
}
/**even given compatible names and types, distinct inline types are distinguished.
the software makes no attempt to unify the types, because that would violate the
principle of least surprise.

for type unification, use a reference.
*/
type DefinedInlineFoo = u64;
type Bat = i64;
///this object is defined inline within `OuterStruct`
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
pub struct DefinedInline {
    /**even given compatible names and types, distinct inline types are distinguished.
the software makes no attempt to unify the types, because that would violate the
principle of least surprise.

for type unification, use a reference.
*/
    pub foo: DefinedInlineFoo,
    pub bat: Bat,
}
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
pub struct OuterStruct {
    ///this object is defined separately, intended to be used within a reference
    pub inner: InnerStruct,
    ///this object is defined inline within `OuterStruct`
    pub defined_inline: DefinedInline,
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {}

