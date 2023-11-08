#![allow(non_camel_case_types)]
///an identifier for an item
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct Id(pub openapi_gen::reexport::uuid::Uuid);
openapi_gen::newtype_derive_canonical_form!(Id, openapi_gen::reexport::uuid::Uuid);

/// An item's status
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde_enum_str::Serialize_enum_str,
    openapi_gen::reexport::serde_enum_str::Deserialize_enum_str,
    Eq,
    Hash
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub enum Status {
    #[serde(rename = "ONE")]
    One,
    #[serde(rename = "TWO")]
    Two,
    #[serde(rename = "THREE")]
    Three,
    #[serde(other)]
    Other(String),
}
type Foo = String;
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
pub struct Item {
    ///an identifier for an item
    pub id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foo: Option<Foo>,

    /// An item's status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct XRequestId(pub openapi_gen::reexport::uuid::Uuid);
openapi_gen::newtype_derive_canonical_form!(
    XRequestId, openapi_gen::reexport::uuid::Uuid
);
///Combination item for query parameters of `getList`
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
pub struct GetListQueryParameters {

    /// An item's status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
    ///an identifier for an item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
}
type Ok_ = Vec<Item>;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq
)]
#[serde(crate = "openapi_gen::reexport::serde", tag = "status")]
pub enum GetListResponse {
    #[serde(rename = "OK")]
    Ok(Ok_),
    Default(Default_),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {

    /// Get a list of natural person identifications.
    /// 
    /// ## Endpoint Data
    /// 
    /// `GET /list`
    /// 
    /// Operation ID: `getList`
    async fn get_list(
        &self,
        status: Option<Status>,
        id: Option<Id>,
        x_request_id: Option<XRequestId>,
    ) -> GetListResponse;
}
impl openapi_gen::reexport::headers::Header for XRequestId {
    fn name() -> &'static openapi_gen::reexport::headers::HeaderName {
        static NAME: openapi_gen::reexport::headers::HeaderName = openapi_gen::reexport::headers::HeaderName::from_static(
            "x-request-id",
        );
        &NAME
    }
    fn decode<'i, I>(
        values: &mut I,
    ) -> Result<Self, openapi_gen::reexport::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i openapi_gen::reexport::headers::HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(openapi_gen::reexport::headers::Error::invalid)?;
        let value_str = value
            .to_str()
            .map_err(|_| openapi_gen::reexport::headers::Error::invalid())?;
        openapi_gen::CanonicalForm::validate(value_str)
            .map_err(|_| openapi_gen::reexport::headers::Error::invalid())
    }
    fn encode<E>(&self, values: &mut E)
    where
        E: ::std::iter::Extend<openapi_gen::reexport::headers::HeaderValue>,
    {
        let value = openapi_gen::CanonicalForm::canonicalize(self)
            .expect("header encoding must be infallible");
        let header_value = openapi_gen::reexport::headers::HeaderValue::from_str(&value)
            .expect("header canonical form must include only visible ascii");
        values.extend(::std::iter::once(header_value));
    }
}
impl openapi_gen::reexport::axum::response::IntoResponse for GetListResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            GetListResponse::Ok(ok) => {
                let mut header_map = openapi_gen::reexport::http::header::HeaderMap::with_capacity(
                    1usize,
                );
                header_map
                    .insert(
                        openapi_gen::reexport::http::header::CONTENT_TYPE,
                        openapi_gen::reexport::http::HeaderValue::from_static(
                            "application/json",
                        ),
                    );
                (
                    openapi_gen::reexport::http::status::StatusCode::OK,
                    header_map,
                    openapi_gen::reexport::axum::Json(ok),
                )
                    .into_response()
            }
            GetListResponse::Default(default) => {
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
            "/list",
            openapi_gen::reexport::axum::routing::get({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Query(
                        GetListQueryParameters { status, id },
                    ): openapi_gen::reexport::axum::extract::Query<
                        GetListQueryParameters,
                    >,
                    x_request_id: Option<
                        openapi_gen::reexport::axum::extract::TypedHeader<XRequestId>,
                    >|
                async move {
                    let x_request_id = x_request_id.map(|x_request_id| x_request_id.0);
                    instance.get_list(status, id, x_request_id).await
                }
            }),
        )
}

