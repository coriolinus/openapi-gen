#![allow(non_camel_case_types)]
///Combination item for query parameters of `getRoot`
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct GetRootQueryParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bar: Option<u64>,
    pub bat: openapi_gen::reexport::uuid::Uuid,
    #[serde(rename = "camelCaseName")]
    pub camel_case_name: String,
}
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum GetRootResponse {
    #[serde(rename = "OK")]
    Ok(Vec<u8>),
    Default(openapi_gen::reexport::http_api_problem::HttpApiProblem),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {

    /// `GET /`
    /// 
    /// Operation ID: `getRoot`
    async fn get_root(
        &self,
        bar: Option<u64>,
        bat: openapi_gen::reexport::uuid::Uuid,
        camel_case_name: String,
    ) -> GetRootResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for GetRootResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            GetRootResponse::Ok(ok) => {
                let mut header_map = openapi_gen::reexport::http::header::HeaderMap::with_capacity(
                    1usize,
                );
                header_map
                    .insert(
                        openapi_gen::reexport::http::header::CONTENT_TYPE,
                        openapi_gen::reexport::http::HeaderValue::from_static(
                            "application/octet-stream",
                        ),
                    );
                (openapi_gen::reexport::http::status::StatusCode::OK, header_map, ok)
                    .into_response()
            }
            GetRootResponse::Default(default) => {
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
            "/",
            openapi_gen::reexport::axum::routing::get({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Query(
                        GetRootQueryParameters { bar, bat, camel_case_name },
                    ): openapi_gen::reexport::axum::extract::Query<
                        GetRootQueryParameters,
                    >|
                async move { instance.get_root(bar, bat, camel_case_name).await }
            }),
        )
}

