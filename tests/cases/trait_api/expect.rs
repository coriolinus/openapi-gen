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
#[serde(untagged)]
pub enum PostKudosResponse {
    Created(Created),
    Default(Default_),
}
#[openapi_gen::reexports::async_trait::async_trait]
pub trait Api {
    /**`POST /post-kudos`

Operation ID: `postKudos`

*/
    async fn post_kudos(request_body: PostKudosRequest) -> PostKudosResponse;
}

