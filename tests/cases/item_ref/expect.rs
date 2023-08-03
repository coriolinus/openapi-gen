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
pub struct Id(pub openapi_gen::reexport::uuid::Uuid);
type Foo = f64;
type Bar = String;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct Thing {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foo: Option<Foo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar: Option<Bar>,
}
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum GetThingResponse {
    #[serde(rename = "OK")]
    Ok(Thing),
}
pub type PutThingRequest = Thing;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum PutThingResponse {
    #[serde(rename = "OK")]
    Ok(Thing),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /**`GET /thing/{id}`

Operation ID: `getThing`

*/
    async fn get_thing(&self, id: Id) -> GetThingResponse;
    /**`PUT /thing/{id}`

Operation ID: `putThing`

*/
    async fn put_thing(&self, id: Id, request_body: PutThingRequest) -> PutThingResponse;
}

