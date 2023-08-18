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
///the FINVIA PersonId for this individual
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
pub struct PersonId(pub openapi_gen::reexport::uuid::Uuid);
openapi_gen::newtype_derive_canonical_form!(PersonId, openapi_gen::reexport::uuid::Uuid);
type Id = IdentificationId;
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
pub struct NaturalPersonIdentification {
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    ///the FINVIA PersonId for this individual
    pub person_id: PersonId,
}
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::From,
    openapi_gen::reexport::derive_more::Into,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct XFlowId(openapi_gen::reexport::uuid::Uuid);
openapi_gen::newtype_derive_canonical_form!(XFlowId, openapi_gen::reexport::uuid::Uuid);
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::From,
    openapi_gen::reexport::derive_more::Into,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct XRequestId(openapi_gen::reexport::uuid::Uuid);
openapi_gen::newtype_derive_canonical_form!(
    XRequestId, openapi_gen::reexport::uuid::Uuid
);
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
/**The path at which the new resource can be found.

This is absolute in that it starts with a `/`, and describes the complete path.
It is relative in that it does not name the server, transport method, port, or other URL-specific data.
*/
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
pub struct Location(pub String);
openapi_gen::newtype_derive_canonical_form!(Location, String);
pub type CreateNaturalPersonIdentificationRequest = NaturalPersonIdentification;
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
pub struct CreateNaturalPersonIdentificationResponseCreated {
    /**The path at which the new resource can be found.

This is absolute in that it starts with a `/`, and describes the complete path.
It is relative in that it does not name the server, transport method, port, or other URL-specific data.
*/
    pub location: Location,
    pub body: NaturalPersonIdentification,
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
pub enum CreateNaturalPersonIdentificationResponse {
    Created(CreateNaturalPersonIdentificationResponseCreated),
    Default(Default_),
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {
    /**Create a new natural person identification.

An identification is a snapshot in time of the data available to identify a person.


## Endpoint Data

`POST /natural-persons`

Operation ID: `createNaturalPersonIdentification`

*/
    async fn create_natural_person_identification(
        &self,
        x_flow_id: Option<XFlowId>,
        x_request_id: Option<XRequestId>,
        request_body: CreateNaturalPersonIdentificationRequest,
    ) -> CreateNaturalPersonIdentificationResponse;
}
impl openapi_gen::reexport::headers::Header for XFlowId {
    fn name() -> &'static openapi_gen::reexport::headers::HeaderName {
        static NAME: openapi_gen::reexport::headers::HeaderName = openapi_gen::reexport::headers::HeaderName::from_static(
            "x-flow-id",
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
impl openapi_gen::reexport::headers::Header for Location {
    fn name() -> &'static openapi_gen::reexport::headers::HeaderName {
        static NAME: openapi_gen::reexport::headers::HeaderName = openapi_gen::reexport::headers::HeaderName::from_static(
            "location",
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
impl openapi_gen::reexport::axum::response::IntoResponse
for CreateNaturalPersonIdentificationResponse {
    fn into_response(self) -> openapi_gen::reexport::axum::response::Response {
        match self {
            CreateNaturalPersonIdentificationResponse::Created(created) => {
                let CreateNaturalPersonIdentificationResponseCreated {
                    location,
                    body,
                } = created;
                let mut header_map = openapi_gen::reexport::http::header::HeaderMap::with_capacity(
                    1usize,
                );
                header_map
                    .insert(
                        openapi_gen::reexport::http::header::HeaderName::from_static(
                            "location",
                        ),
                        openapi_gen::header_value_of!(& location),
                    );
                (
                    openapi_gen::reexport::http::status::StatusCode::CREATED,
                    header_map,
                    openapi_gen::reexport::axum::Json(body),
                )
                    .into_response()
            }
            CreateNaturalPersonIdentificationResponse::Default(default) => {
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
            "/natural-persons",
            openapi_gen::reexport::axum::routing::post({
                let instance = instance.clone();
                move |
                    x_flow_id: Option<
                        openapi_gen::reexport::axum::extract::TypedHeader<XFlowId>,
                    >,
                    x_request_id: Option<
                        openapi_gen::reexport::axum::extract::TypedHeader<XRequestId>,
                    >,
                    openapi_gen::reexport::axum::extract::Json(
                        request_body,
                    ): openapi_gen::reexport::axum::extract::Json<
                        CreateNaturalPersonIdentificationRequest,
                    >|
                async move {
                    let x_flow_id = x_flow_id.map(|x_flow_id| x_flow_id.0);
                    let x_request_id = x_request_id.map(|x_request_id| x_request_id.0);
                    instance
                        .create_natural_person_identification(
                            x_flow_id,
                            x_request_id,
                            request_body,
                        )
                        .await
                }
            }),
        )
}

