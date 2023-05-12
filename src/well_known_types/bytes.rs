use base64::{engine::general_purpose::STANDARD, Engine};

use crate::{CanonicalForm, Reason, ValidationError};

#[derive(
    Debug,
    derive_more::Constructor,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Bytes(pub Vec<u8>);

impl CanonicalForm for Bytes {
    type JsonRepresentation = String;

    fn validate(from: &Self::JsonRepresentation) -> Result<Self, ValidationError> {
        STANDARD
            .decode(from)
            .map(Self)
            .map_err(|err| ValidationError::reason::<Self>(Reason::from_err(err)))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, crate::CanonicalizeError> {
        Ok(STANDARD.encode(&self.0))
    }
}
