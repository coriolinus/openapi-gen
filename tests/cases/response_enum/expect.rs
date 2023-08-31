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
    /// `POST /render`
    /// 
    /// Operation ID: `renderPdf`

    async fn render_pdf(&self) -> RenderPdfResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for RenderPdfResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            RenderPdfResponse::Ok(ok) => {
                let mut header_map = openapi_gen::reexport::http::header::HeaderMap::with_capacity(
                    1usize,
                );
                header_map
                    .insert(
                        openapi_gen::reexport::http::header::CONTENT_TYPE,
                        openapi_gen::reexport::http::HeaderValue::from_static(
                            "application/pdf",
                        ),
                    );
                (openapi_gen::reexport::http::status::StatusCode::OK, header_map, ok)
                    .into_response()
            }
            RenderPdfResponse::BadRequest(bad_request) => {
                (
                    openapi_gen::reexport::http::status::StatusCode::BAD_REQUEST,
                    openapi_gen::reexport::axum::Json(bad_request),
                )
                    .into_response()
            }
            RenderPdfResponse::ServiceUnavailable(service_unavailable) => {
                (
                    openapi_gen::reexport::http::status::StatusCode::SERVICE_UNAVAILABLE,
                    openapi_gen::reexport::axum::Json(service_unavailable),
                )
                    .into_response()
            }
            RenderPdfResponse::Default(default) => {
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
            "/render",
            openapi_gen::reexport::axum::routing::post({
                let instance = instance.clone();
                move || async move { instance.render_pdf().await }
            }),
        )
}

