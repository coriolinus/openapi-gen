#[cfg(feature = "integer-restrictions")]
pub(crate) mod bounded_integer;
pub(crate) mod numbers;
pub(crate) mod time;

use crate::{CanonicalForm, CanonicalizeError, Reason, ValidationError};

macro_rules! canonical_form_transparent {
    ($t:ty) => {
        impl CanonicalForm for $t {
            type JsonRepresentation = $t;
            fn validate(from: &Self::JsonRepresentation) -> Result<Self, ValidationError> {
                Ok(from.to_owned())
            }
            fn canonicalize(&self) -> Result<$t, CanonicalizeError> {
                Ok(self.clone())
            }
        }
    };
}

canonical_form_transparent!(bool);
canonical_form_transparent!(String);

macro_rules! canonical_form_display_fromstr {
    ($t:path) => {
        impl CanonicalForm for $t {
            type JsonRepresentation = String;
            fn validate(from: &String) -> Result<Self, ValidationError> {
                from.parse()
                    .map_err(|err| ValidationError::reason::<Self>(Reason::from_err(err)))
            }
            fn canonicalize(&self) -> Result<String, CanonicalizeError> {
                Ok(self.to_string())
            }
        }
    };
}

canonical_form_display_fromstr!(std::net::Ipv4Addr);
canonical_form_display_fromstr!(std::net::Ipv6Addr);
canonical_form_display_fromstr!(std::net::IpAddr);

#[cfg(feature = "uuid")]
canonical_form_display_fromstr!(uuid::Uuid);
