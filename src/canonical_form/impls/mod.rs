#[cfg(feature = "integer-restrictions")]
pub(crate) mod bounded_integer;
pub(crate) mod numbers;
pub(crate) mod time;

use crate::{CanonicalForm, CanonicalizeError, Reason, ValidationError};

macro_rules! canonical_form_transparent {
    ($json_repr:ty) => {
        canonical_form_transparent!($json_repr, $json_repr);
    };
    ($json_repr:ty, $parseable_from:ty) => {
        impl CanonicalForm for $json_repr {
            type ParseableFrom = $parseable_from;
            type JsonRepresentation = $json_repr;
            fn validate(from: &$parseable_from) -> Result<Self, ValidationError> {
                Ok(from.to_owned())
            }
            fn canonicalize(&self) -> Result<$json_repr, CanonicalizeError> {
                Ok(self.clone())
            }
        }
    };
}

canonical_form_transparent!(bool);
canonical_form_transparent!(String, str);

macro_rules! canonical_form_display_fromstr {
    ($t:path) => {
        impl CanonicalForm for $t {
            type ParseableFrom = str;
            type JsonRepresentation = String;
            fn validate(from: &str) -> Result<Self, ValidationError> {
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

impl CanonicalForm for serde_json::Value {
    type ParseableFrom = str;
    type JsonRepresentation = String;

    fn validate(from: &Self::ParseableFrom) -> Result<Self, ValidationError> {
        serde_json::from_str(from)
            .map_err(|err| ValidationError::reason::<Self>(Reason::from_err(err)))
    }

    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError> {
        Ok(self.to_string())
    }
}
