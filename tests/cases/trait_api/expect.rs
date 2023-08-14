#![allow(non_camel_case_types)]
pub type PostKudo = openapi_gen::reexport::serde_json::Value;
///request body for a freeform render request
pub type PostKudosRequest = PostKudo;
type Created = ();
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
pub enum PostKudosResponse {
    Created(Created),
    Default(Default_),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /**`POST /post-kudos`

Operation ID: `postKudos`

*/
    async fn post_kudos(&self, request_body: PostKudosRequest) -> PostKudosResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for PostKudosResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            PostKudosResponse::Created(created) => {
                (openapi_gen::reexport::http::status::StatusCode::CREATED, created)
                    .into_response()
            }
            PostKudosResponse::Default(default) => {
                openapi_gen::axum_compat::default_response(default)
            }
        }
    }
}

