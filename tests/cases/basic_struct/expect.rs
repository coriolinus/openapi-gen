///unsigned integer
type Foo = u64;
type Bar = String;
type MaybeBar = Option<String>;
///this object is defined separately, intended to be used within a reference
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor,
)]
pub struct InnerStruct {
    ///unsigned integer
    foo: Foo,
    bar: MaybeBar,
}
/**even given compatible names and types, distinct inline types are distinguished.
the software makes no attempt to unify the types, because that would violate the
principle of least surprise.

for type unification, use a reference.
*/
type Foo1 = u64;
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
    openapi_gen::reexport::derive_more::Constructor,
)]
pub struct DefinedInline {
    /**even given compatible names and types, distinct inline types are distinguished.
    the software makes no attempt to unify the types, because that would violate the
    principle of least surprise.

    for type unification, use a reference.
    */
    foo: Foo1,
    bat: Bat,
}
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor,
)]
pub struct OuterStruct {
    ///this object is defined separately, intended to be used within a reference
    inner: InnerStruct,
    ///this object is defined inline within `OuterStruct`
    defined_inline: DefinedInline,
}
