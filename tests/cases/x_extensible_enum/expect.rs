#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash
)]
pub enum DeliveryMethod {
    Parcel,
    Letter,
    Email,
    Other(String),
}

