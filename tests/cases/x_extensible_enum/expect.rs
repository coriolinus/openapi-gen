#![allow(non_camel_case_types)]
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

