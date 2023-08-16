use serde_json::Number;

use crate::{CanonicalForm, CanonicalizeError, Reason, ValidationError};

impl CanonicalForm for f64 {
    type ParseableFrom = Number;
    type JsonRepresentation = Number;

    fn validate(from: &Self::ParseableFrom) -> Result<Self, ValidationError> {
        from.as_f64()
            .ok_or_else(|| ValidationError::reason::<Self>(format!("number not finite: {from}")))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        Number::from_f64(*self)
            .ok_or_else(|| CanonicalizeError::reason::<Self>(format!("number not finite: {self}")))
    }
}

impl CanonicalForm for f32 {
    type ParseableFrom = Number;
    type JsonRepresentation = Number;

    fn validate(from: &Self::ParseableFrom) -> Result<Self, ValidationError> {
        f64::validate(from).map(|n| n as _)
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        (*self as f64).canonicalize()
    }
}

impl CanonicalForm for i64 {
    type ParseableFrom = Number;
    type JsonRepresentation = Number;

    fn validate(from: &Self::ParseableFrom) -> Result<Self, ValidationError> {
        from.as_i64()
            .ok_or_else(|| ValidationError::reason::<Self>(format!("number not integer: {from}")))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        Ok((*self).into())
    }
}

impl CanonicalForm for i32 {
    type ParseableFrom = Number;
    type JsonRepresentation = Number;

    fn validate(from: &Self::ParseableFrom) -> Result<Self, ValidationError> {
        i64::validate(from).and_then(|n| {
            n.try_into()
                .map_err(|err| ValidationError::reason::<Self>(Reason::from_err(err)))
        })
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        i64::from(*self).canonicalize()
    }
}
