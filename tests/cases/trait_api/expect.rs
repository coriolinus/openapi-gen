#![allow(non_camel_case_types)]
pub type PostKudo = openapi_gen::reexport::serde_json::Value;
///request body for a freeform render request
pub type PostKudosRequest = PostKudo;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum PostKudosResponse {
    Created(()),
    Default(openapi_gen::reexport::http_api_problem::HttpApiProblem),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {

    /// `POST /post-kudos`
    /// 
    /// Operation ID: `postKudos`
    async fn post_kudos(&self, request_body: PostKudosRequest) -> PostKudosResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for PostKudosResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            PostKudosResponse::Created(created) => {
                (openapi_gen::reexport::http::status::StatusCode::CREATED, created)
                    .into_response()
            }
            PostKudosResponse::Default(default) => default.into_response(),
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
            "/post-kudos",
            openapi_gen::reexport::axum::routing::post({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum_extra::extract::WithRejection(
                        openapi_gen::reexport::axum::extract::Json(request_body),
                        _,
                    ): openapi_gen::reexport::axum_extra::extract::WithRejection<
                        openapi_gen::reexport::axum::extract::Json<PostKudosRequest>,
                        openapi_gen::axum_compat::ApiProblemRejection,
                    >|
                async move { instance.post_kudos(request_body).await }
            }),
        )
}

