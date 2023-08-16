use serde::{Deserialize, Serialize};

use crate::CanonicalForm;

#[cfg(feature = "axum-support")]
use axum::headers::Header;

/// This header lets the client specify what sort of content it wants to receive.
///
/// It is automatically injected into an endpoint's parameter list when there is more than one content-type permissible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accept {
    // private field so end users can't just construct this
    _pd: std::marker::PhantomData<()>,
} // TODO: build the struct

impl CanonicalForm for Accept {
    type ParseableFrom = str;
    type JsonRepresentation = String;

    fn validate(from: &Self::ParseableFrom) -> Result<Self, crate::ValidationError> {
        todo!()
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, crate::CanonicalizeError> {
        todo!()
    }
}

#[cfg(feature = "axum-support")]
impl Header for Accept {
    fn name() -> &'static http::HeaderName {
        static NAME: http::HeaderName = http::HeaderName::from_static("accept");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i http::HeaderValue>,
    {
        let value = values.next().ok_or_else(axum::headers::Error::invalid)?;
        let value_str = value
            .to_str()
            .map_err(|_| axum::headers::Error::invalid())?;
        CanonicalForm::validate(value_str).map_err(|_| axum::headers::Error::invalid())
    }

    fn encode<E: Extend<http::HeaderValue>>(&self, values: &mut E) {
        let value = CanonicalForm::canonicalize(self).expect("header encoding must be infallible");
        let header_value = axum::headers::HeaderValue::from_str(&value)
            .expect("header canonical form must include only visible ascii");
        values.extend(::std::iter::once(header_value));
    }
}
