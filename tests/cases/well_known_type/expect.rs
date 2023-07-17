pub type PostWellKnownTypesRequest = openapi_gen::reexport::serde_json::Value;
type NoContent = ();
type Default = openapi_gen::reexport::http_api_problem::HttpApiProblem;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(untagged)]
pub enum PostWellKnownTypesResponse {
    NoContent(NoContent),
    Default(Default),
}

