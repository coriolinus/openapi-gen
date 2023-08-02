#![allow(non_camel_case_types)]
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
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum RenderPdfResponse {
    #[serde(rename = "OK")]
    Ok(Ok_),
    #[serde(rename = "Bad Request")]
    BadRequest(RenderError),
    #[serde(rename = "Service Unavailable")]
    ServiceUnavailable(ServiceUnavailable),
    Default(Default_),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /**`POST /render`

Operation ID: `renderPdf`

*/
    async fn render_pdf() -> RenderPdfResponse;
}

