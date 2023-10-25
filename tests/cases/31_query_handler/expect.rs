#![allow(non_camel_case_types)]
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
type Bar = u64;
type Bat = openapi_gen::reexport::uuid::Uuid;
type Ok_ = Vec<u8>;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
)]
#[serde(crate = "openapi_gen::reexport::serde")]
struct GetRootQueryParameters {
    bar: Option<Bar>,
    bat: Bat,
}
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum GetRootResponse {
    #[serde(rename = "OK")]
    Ok(Ok_),
    Default(Default_),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /// `GET /`
    ///
    /// Operation ID: `getRoot`
    async fn get_root(&self, bar: Option<Bar>, bat: Bat) -> GetRootResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse for GetRootResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            GetRootResponse::Ok(ok) => {
                let mut header_map =
                    openapi_gen::reexport::http::header::HeaderMap::with_capacity(1usize);
                header_map.insert(
                    openapi_gen::reexport::http::header::CONTENT_TYPE,
                    openapi_gen::reexport::http::HeaderValue::from_static(
                        "application/octet-stream",
                    ),
                );
                (
                    openapi_gen::reexport::http::status::StatusCode::OK,
                    header_map,
                    ok,
                )
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
    openapi_gen::reexport::axum::Router::new().route(
        "/",
        openapi_gen::reexport::axum::routing::get({
            let instance = instance.clone();
            move |openapi_gen::reexport::axum::extract::Query(GetRootQueryParameters {
                      bar,
                      bat,
                  }): openapi_gen::reexport::axum::extract::Query<
                GetRootQueryParameters,
            >| async move { instance.get_root(bar, bat).await }
        }),
    )
}
