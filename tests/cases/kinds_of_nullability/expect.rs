#![allow(non_camel_case_types)]
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
pub struct Foo {
    pub not_nullable_and_required: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_nullable_and_not_required: Option<i64>,
    pub nullable_and_required: Option<i64>,
    ///note that this produces an `Option<Option<_>>`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable_and_not_required: Option<Option<i64>>,
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

