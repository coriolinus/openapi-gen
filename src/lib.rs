pub(crate) mod as_status_code;
pub(crate) mod canonical_form;
pub(crate) mod codegen;
pub(crate) mod openapi_compat;
pub(crate) mod resolve_trait;
pub(crate) mod well_known_types;

pub use as_status_code::AsStatusCode;

pub use canonical_form::{
    CanonicalForm, CanonicalizeError, ConstraintViolation, Reason, ValidationError,
};

pub use codegen::{ApiModel, Error};

#[cfg(feature = "bytes")]
pub use well_known_types::Bytes;

/// Reexport crates used by generated code.
///
/// This makes it much easier to keep the types in sync between the generated code and your own types.
pub mod reexport {
    pub use async_trait;
    #[cfg(feature = "integer-restrictions")]
    pub use bounded_integer;
    pub use derive_more;
    pub use heck;
    #[cfg(feature = "api-problem")]
    pub use http_api_problem;
    #[cfg(feature = "string-pattern")]
    pub use regress;
    pub use serde;
    pub use serde_enum_str;
    pub use serde_json;
    pub use time;
    #[cfg(feature = "uuid")]
    pub use uuid;
}
