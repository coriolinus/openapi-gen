#![allow(non_camel_case_types)]
///should be `pub type Count = u64`
pub type Count = u64;
///should be `pub struct FirstBar(pub String);`
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::From,
    openapi_gen::reexport::derive_more::Into,
    openapi_gen::reexport::derive_more::Deref,
    openapi_gen::reexport::derive_more::DerefMut,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct FirstBar(pub String);
openapi_gen::newtype_derive_canonical_form!(FirstBar, String);
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
pub struct Foo {
    ///should be `pub type Count = u64`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty_of_bar: Option<Count>,
    ///should be `pub struct FirstBar(pub String);`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_bar: Option<FirstBar>,
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

