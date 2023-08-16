use std::{fmt, str::FromStr};

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::{CanonicalForm, Reason, ValidationError};

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    serde_with::SerializeDisplay,
    serde_with::DeserializeFromStr,
    derive_more::Constructor,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Bytes(pub Vec<u8>);

impl CanonicalForm for Bytes {
    type ParseableFrom = str;
    type JsonRepresentation = String;

    fn validate(from: &str) -> Result<Self, ValidationError> {
        from.parse()
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, crate::CanonicalizeError> {
        Ok(STANDARD.encode(&self.0))
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            &self
                .canonicalize()
                .expect("bytes canonicalization never fails"),
        )
    }
}

impl FromStr for Bytes {
    type Err = ValidationError;

    fn from_str(from: &str) -> Result<Self, Self::Err> {
        STANDARD
            .decode(from)
            .map(Self)
            .map_err(|err| ValidationError::reason::<Self>(Reason::from_err(err)))
    }
}
