#![allow(non_camel_case_types)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde_enum_str::Serialize_enum_str,
    openapi_gen::reexport::serde_enum_str::Deserialize_enum_str,
    Eq,
    Hash
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub enum DeliveryMethod {
    #[serde(rename = "parcel")]
    Parcel,
    #[serde(rename = "letter")]
    Letter,
    #[serde(rename = "email")]
    Email,
    #[serde(other)]
    Other(String),
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

