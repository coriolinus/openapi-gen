#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub enum DeliveryMethod {
    Parcel,
    Letter,
    Email,
    Other(String),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {}

