use crate::{CanonicalForm, CanonicalizeError, Reason, ValidationError};
use time::{
    format_description::{well_known::Rfc3339, FormatItem},
    macros::format_description,
    Date, OffsetDateTime,
};

const DATE_FORM: &[FormatItem<'static>] = format_description!("[year]-[month]-[day]");

/// The canonical form for a `Date` is `YYYY-MM-DD`.
///
/// It is defiend by [RFC 3339, section 5.6].
///
/// [RFC 3339, section 5.6]: https://tools.ietf.org/html/rfc3339#section-5.6
impl CanonicalForm for Date {
    type JsonRepresentation = String;

    fn validate(from: &Self::JsonRepresentation) -> Result<Self, ValidationError> {
        Self::parse(from, &DATE_FORM)
            .map_err(|err| ValidationError::reason::<Self>(Reason::from_err(err)))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        Ok(self
            .format(&DATE_FORM)
            .expect("`Date` type cannot fail to format with with this format"))
    }
}

/// The canonical form for an `OffsetDateTIme` is `YYYY-MM-DDThh:mm:ssZ`.
///
/// It is defiend by [RFC 3339, section 5.6].
///
/// [RFC 3339, section 5.6]: https://tools.ietf.org/html/rfc3339#section-5.6
impl CanonicalForm for OffsetDateTime {
    type JsonRepresentation = String;

    fn validate(from: &Self::JsonRepresentation) -> Result<Self, ValidationError> {
        Self::parse(from, &Rfc3339)
            .map_err(|err| ValidationError::reason::<Self>(Reason::from_err(err)))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        Ok(self
            .format(&Rfc3339)
            .expect("`OffsetDateTime` type cannot fail to format with `Rfc3339`"))
    }
}
