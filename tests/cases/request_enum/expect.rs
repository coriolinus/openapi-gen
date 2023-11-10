#![allow(non_camel_case_types)]
pub type JsonType = openapi_gen::reexport::serde_json::Value;
pub type FormType = openapi_gen::reexport::serde_json::Value;
pub type ReqType = openapi_gen::reexport::serde_json::Value;
pub type ApplicationJson = JsonType;
pub type MultipartFormValue = openapi_gen::reexport::serde_json::Value;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize
)]
#[serde(crate = "openapi_gen::reexport::serde", untagged)]
pub enum MultiRequestsRequest {
    ApplicationJson(ApplicationJson),
    MultipartFormValue(MultipartFormValue),
}
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum MultiRequestsResponse {
    Default(openapi_gen::reexport::http_api_problem::HttpApiProblem),
}
///request body is optional
pub type OptionalRequestBodyRequest = Option<JsonType>;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum OptionalRequestBodyResponse {
    Default(openapi_gen::reexport::http_api_problem::HttpApiProblem),
}
pub type SameRequestRequest = ReqType;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum SameRequestResponse {
    Default(openapi_gen::reexport::http_api_problem::HttpApiProblem),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {

    /// `POST /multi-requests`
    /// 
    /// Operation ID: `multiRequests`
    async fn multi_requests(
        &self,
        request_body: MultiRequestsRequest,
    ) -> MultiRequestsResponse;

    /// `POST /optional-request-body`
    /// 
    /// Operation ID: `optionalRequestBody`
    async fn optional_request_body(
        &self,
        request_body: OptionalRequestBodyRequest,
    ) -> OptionalRequestBodyResponse;

    /// `POST /unified-request-body`
    /// 
    /// Operation ID: `sameRequest`
    async fn same_request(
        &self,
        request_body: SameRequestRequest,
    ) -> SameRequestResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for MultiRequestsResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            MultiRequestsResponse::Default(default) => {
                openapi_gen::axum_compat::default_response(default)
            }
        }
    }
}
impl openapi_gen::reexport::axum::response::IntoResponse
for OptionalRequestBodyResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            OptionalRequestBodyResponse::Default(default) => {
                openapi_gen::axum_compat::default_response(default)
            }
        }
    }
}
impl openapi_gen::reexport::axum::response::IntoResponse for SameRequestResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            SameRequestResponse::Default(default) => {
                openapi_gen::axum_compat::default_response(default)
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
            "/multi-requests",
            openapi_gen::reexport::axum::routing::post({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Json(
                        request_body,
                    ): openapi_gen::reexport::axum::extract::Json<MultiRequestsRequest>|
                async move { instance.multi_requests(request_body).await }
            }),
        )
        .route(
            "/optional-request-body",
            openapi_gen::reexport::axum::routing::post({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Json(
                        request_body,
                    ): openapi_gen::reexport::axum::extract::Json<
                        OptionalRequestBodyRequest,
                    >|
                async move { instance.optional_request_body(request_body).await }
            }),
        )
        .route(
            "/unified-request-body",
            openapi_gen::reexport::axum::routing::post({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Json(
                        request_body,
                    ): openapi_gen::reexport::axum::extract::Json<SameRequestRequest>|
                async move { instance.same_request(request_body).await }
            }),
        )
}

