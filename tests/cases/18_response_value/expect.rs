#![allow(non_camel_case_types)]
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
type AdditionalInformationItem = String;
pub type AdditionalInformation = Vec<AdditionalInformationItem>;
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
    pub person_id: PersonId,
}
type Default_ = openapi_gen::reexport::http_api_problem::HttpApiProblem;
pub type Location = String;
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
    #[serde(rename = "Location")]
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
    /**`POST /natural-persons`

Operation ID: `createNaturalPersonIdentification`

*/
    async fn create_natural_person_identification(
        &self,
        request_body: CreateNaturalPersonIdentificationRequest,
    ) -> CreateNaturalPersonIdentificationResponse;
}

