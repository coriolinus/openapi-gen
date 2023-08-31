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
openapi_gen::newtype_derive_canonical_form!(Id, openapi_gen::reexport::uuid::Uuid);
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

    /// `GET /thing/{id}`
    /// 
    /// Operation ID: `getThing`
    async fn get_thing(&self, id: Id) -> GetThingResponse;

    /// `PUT /thing/{id}`
    /// 
    /// Operation ID: `putThing`
    async fn put_thing(&self, id: Id, request_body: PutThingRequest) -> PutThingResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for GetThingResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            GetThingResponse::Ok(ok) => {
                (
                    openapi_gen::reexport::http::status::StatusCode::OK,
                    openapi_gen::reexport::axum::Json(ok),
                )
                    .into_response()
            }
        }
    }
}
impl openapi_gen::reexport::axum::response::IntoResponse for PutThingResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            PutThingResponse::Ok(ok) => {
                (
                    openapi_gen::reexport::http::status::StatusCode::OK,
                    openapi_gen::reexport::axum::Json(ok),
                )
                    .into_response()
            }
        }
    }
}
/// Transform an instance of [`trait Api`][Api] into a [`Router`][axum::Router].
pub fn build_router<Instance>(instance: Instance) -> openapi_gen::reexport::axum::Router
where
    Instance: 'static + Api + Send + Sync,
{
    #[allow(unused_variables)]
    let instance = ::std::sync::Arc::new(instance);
    openapi_gen::reexport::axum::Router::new()
        .route(
            "/thing/:id",
            openapi_gen::reexport::axum::routing::get({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Path(
                        id,
                    ): openapi_gen::reexport::axum::extract::Path<Id>|
                async move { instance.get_thing(id).await }
            }),
        )
        .route(
            "/thing/:id",
            openapi_gen::reexport::axum::routing::put({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Path(
                        id,
                    ): openapi_gen::reexport::axum::extract::Path<Id>,
                    openapi_gen::reexport::axum::extract::Json(
                        request_body,
                    ): openapi_gen::reexport::axum::extract::Json<PutThingRequest>|
                async move { instance.put_thing(id, request_body).await }
            }),
        )
}

