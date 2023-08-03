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

