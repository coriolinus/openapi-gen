pub type RenderError = openapi_gen::reexport::serde_json::Value;
type Ok_ = Vec<u8>;
type ServiceUnavailable = openapi_gen::reexport::http_api_problem::HttpApiProblem;
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize
)]
#[serde(untagged)]
pub enum RenderPdfResponse {
    Ok(Ok_),
    BadRequest(RenderError),
    ServiceUnavailable(ServiceUnavailable),
    Default(Default_),
}
#[openapi_gen::reexports::async_trait::async_trait]
pub trait Api {
    /**`POST /render`

Operation ID: `renderPdf`

*/
    async fn render_pdf() -> RenderPdfResponse;
}

