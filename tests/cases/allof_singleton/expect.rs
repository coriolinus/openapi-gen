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
pub struct Id(pub openapi_gen::reexport::uuid::Uuid);
type ThingId = Id;
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
pub struct Thing {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<ThingId>,
}
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
pub struct WriteableThing {
    pub id: Id,
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {}

