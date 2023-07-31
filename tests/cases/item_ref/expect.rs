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
#[serde(untagged)]
pub enum GetThingResponse {
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
#[serde(untagged)]
pub enum PutThingResponse {
    Ok(Thing),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /**`GET /thing/{id}`

Operation ID: `getThing`

*/
    async fn get_thing(id: Id) -> GetThingResponse;
    /**`PUT /thing/{id}`

Operation ID: `putThing`

*/
    async fn put_thing(id: Id, request_body: PutThingRequest) -> PutThingResponse;
}

