#![allow(non_camel_case_types)]
type NotNullableAndRequired = i64;
type NotNullableAndNotRequired = i64;
type MaybeNullableAndRequired = Option<NullableAndRequired>;
type NullableAndRequired = i64;
type MaybeNullableAndNotRequired = Option<NullableAndNotRequired>;
///note that this produces an `Option<Option<_>>`
type NullableAndNotRequired = i64;
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
    pub not_nullable_and_required: NotNullableAndRequired,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_nullable_and_not_required: Option<NotNullableAndNotRequired>,
    pub nullable_and_required: MaybeNullableAndRequired,
    ///note that this produces an `Option<Option<_>>`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable_and_not_required: Option<MaybeNullableAndNotRequired>,
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

