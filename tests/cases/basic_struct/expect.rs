#![allow(non_camel_case_types)]
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
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct InnerStruct {
    ///unsigned integer
    pub foo: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar: Option<String>,
}
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
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct DefinedInline {

    /// even given compatible names and types, distinct inline types are distinguished.
    /// the software makes no attempt to unify the types, because that would violate the
    /// principle of least surprise.
    /// 
    /// for type unification, use a reference.
    pub foo: u64,
    pub bat: i64,
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
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct OuterStruct {
    ///this object is defined separately, intended to be used within a reference
    pub inner: InnerStruct,
    ///this object is defined inline within `OuterStruct`
    pub defined_inline: DefinedInline,
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {}
/// Transform an instance of [`trait Api`][Api] into a [`Router`][axum::Router].
pub fn build_router<Instance>(instance: Instance) -> openapi_gen::reexport::axum::Router
where
    Instance: 'static + Api + Send + Sync,
{
    #[allow(unused_variables)]
    let instance = ::std::sync::Arc::new(instance);
    openapi_gen::reexport::axum::Router::new()
}

