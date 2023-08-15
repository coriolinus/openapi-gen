#![allow(non_camel_case_types)]
pub type PostWellKnownTypesRequest = openapi_gen::reexport::serde_json::Value;
type NoContent = ();
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum PostWellKnownTypesResponse {
    #[serde(rename = "No Content")]
    NoContent(NoContent),
    Default(Default_),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /**`POST /well-known-types`

*/
    async fn post_well_known_types(
        &self,
        request_body: PostWellKnownTypesRequest,
    ) -> PostWellKnownTypesResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for PostWellKnownTypesResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            PostWellKnownTypesResponse::NoContent(no_content) => {
                (openapi_gen::reexport::http::status::StatusCode::NO_CONTENT, no_content)
                    .into_response()
            }
            PostWellKnownTypesResponse::Default(default) => {
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
            "/well-known-types",
            openapi_gen::reexport::axum::routing::post({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Json(
                        request_body,
                    ): openapi_gen::reexport::axum::extract::Json<
                        PostWellKnownTypesRequest,
                    >|
                async move { instance.post_well_known_types(request_body).await }
            }),
        )
}

