use std::borrow::Borrow;

use crate::{CanonicalForm, CanonicalizeError, ValidationError};
use bounded_integer::{BoundedI32, BoundedI64};
use serde_json::Number;

impl<const MIN: i64, const MAX: i64> CanonicalForm for BoundedI64<MIN, MAX> {
    type ParseableFrom = Number;
    type JsonRepresentation = Number;

    fn validate(from: &Number) -> Result<Self, ValidationError> {
        let v = i64::validate(from)?;
        BoundedI64::new(v)
            .ok_or_else(|| ValidationError::reason::<Self>(format!("{v} not in {MIN}..={MAX}")))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        self.get().canonicalize()
    }
}

impl<const MIN: i32, const MAX: i32> CanonicalForm for BoundedI32<MIN, MAX> {
    type ParseableFrom = Number;
    type JsonRepresentation = Number;

    fn validate(from: &Number) -> Result<Self, ValidationError> {
        let v = i32::validate(from)?;
        BoundedI32::new(v)
            .ok_or_else(|| ValidationError::reason::<Self>(format!("{v} not in {MIN}..={MAX}")))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        self.get().canonicalize()
    }
}
