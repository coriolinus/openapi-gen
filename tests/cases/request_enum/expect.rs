pub type JsonType = openapi_gen::reexport::serde_json::Value;
pub type FormType = openapi_gen::reexport::serde_json::Value;
pub type ReqType = openapi_gen::reexport::serde_json::Value;
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
///an error occurred; see status code and problem object for more information
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", untagged)]
pub enum Default_1 {
    Default(Default_),
}
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
///request body is optional
pub type OptionalRequestBodyRequest = Option<JsonType>;
pub type SameRequestRequest = ReqType;
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /**`POST /multi-requests`

Operation ID: `multiRequests`

*/
    async fn multi_requests(request_body: MultiRequestsRequest) -> Default_1;
    /**`POST /optional-request-body`

Operation ID: `optionalRequestBody`

*/
    async fn optional_request_body(
        request_body: OptionalRequestBodyRequest,
    ) -> Default_1;
    /**`POST /unified-request-body`

Operation ID: `sameRequest`

*/
    async fn same_request(request_body: SameRequestRequest) -> Default_1;
}

