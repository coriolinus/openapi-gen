#![allow(non_camel_case_types)]
///an identifier for this particular identification process
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
pub struct IdentificationId(pub openapi_gen::reexport::uuid::Uuid);
openapi_gen::newtype_derive_canonical_form!(
    IdentificationId, openapi_gen::reexport::uuid::Uuid
);

/// An identifier for a document within the context of the identification service.
/// 
/// This is _not_ associated with the documents service in any way.
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
pub struct DocumentId(pub openapi_gen::reexport::uuid::Uuid);
openapi_gen::newtype_derive_canonical_form!(
    DocumentId, openapi_gen::reexport::uuid::Uuid
);
///Combination item for path parameters of `getNpIdentityDocumentData`
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
pub struct GetNpIdentityDocumentDataPathParameters {
    ///an identifier for this particular identification process
    #[serde(rename = "identification-id")]
    pub identification_id: IdentificationId,

    /// An identifier for a document within the context of the identification service.
    /// 
    /// This is _not_ associated with the documents service in any way.
    #[serde(rename = "document-id")]
    pub document_id: DocumentId,
}
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
pub struct OkApplicationJson {
    ///document data encoded as base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<openapi_gen::Bytes>,
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
pub enum GetNpIdentityDocumentDataResponse {
    #[serde(rename = "OK application/json")]
    OkApplicationJson(OkApplicationJson),
    #[serde(rename = "OK *")]
    Ok(Vec<u8>),
    #[serde(rename = "Not Acceptable")]
    NotAcceptable(openapi_gen::reexport::http_api_problem::HttpApiProblem),
    Default(openapi_gen::reexport::http_api_problem::HttpApiProblem),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {

    /// Get this identity document's raw data.
    /// 
    /// If the client accepts only `application/json`, then the data will be base64 encoded and enclosed in a small wrapper object.
    /// Otherwise, the actual document content type will be returned, and the document data will be unencoded.
    /// 
    /// 
    /// ## Endpoint Data
    /// 
    /// `GET /natural-persons/{identification-id}/documents/{document-id}/data`
    /// 
    /// Operation ID: `getNpIdentityDocumentData`
    async fn get_np_identity_document_data(
        &self,
        identification_id: IdentificationId,
        document_id: DocumentId,
        accept: Option<openapi_gen::reexport::accept_header::Accept>,
    ) -> GetNpIdentityDocumentDataResponse;
}
impl openapi_gen::reexport::axum::response::IntoResponse
for GetNpIdentityDocumentDataResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            GetNpIdentityDocumentDataResponse::OkApplicationJson(
                ok_application_json,
            ) => {
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
                    openapi_gen::reexport::axum::Json(ok_application_json),
                )
                    .into_response()
            }
            GetNpIdentityDocumentDataResponse::Ok(ok) => {
                let mut header_map = openapi_gen::reexport::http::header::HeaderMap::with_capacity(
                    1usize,
                );
                header_map
                    .insert(
                        openapi_gen::reexport::http::header::CONTENT_TYPE,
                        openapi_gen::reexport::http::HeaderValue::from_static("*"),
                    );
                (openapi_gen::reexport::http::status::StatusCode::OK, header_map, ok)
                    .into_response()
            }
            GetNpIdentityDocumentDataResponse::NotAcceptable(not_acceptable) => {
                (
                    openapi_gen::reexport::http::status::StatusCode::NOT_ACCEPTABLE,
                    openapi_gen::reexport::axum::Json(not_acceptable),
                )
                    .into_response()
            }
            GetNpIdentityDocumentDataResponse::Default(default) => {
                default.into_response()
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
            "/natural-persons/:identification-id/documents/:document-id/data",
            openapi_gen::reexport::axum::routing::get({
                let instance = instance.clone();
                move |
                    openapi_gen::reexport::axum::extract::Path(
                        GetNpIdentityDocumentDataPathParameters {
                            identification_id,
                            document_id,
                        },
                    ): openapi_gen::reexport::axum::extract::Path<
                        GetNpIdentityDocumentDataPathParameters,
                    >,
                    accept: Option<
                        openapi_gen::reexport::axum_extra::TypedHeader<
                            openapi_gen::reexport::accept_header::Accept,
                        >,
                    >|
                async move {
                    let accept = accept.map(|accept| accept.0);
                    instance
                        .get_np_identity_document_data(
                            identification_id,
                            document_id,
                            accept,
                        )
                        .await
                }
            }),
        )
}

