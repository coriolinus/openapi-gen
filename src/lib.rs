pub(crate) mod canonical_form;
pub(crate) mod codegen;
pub(crate) mod openapi_compat;
pub(crate) mod resolve_trait;
pub(crate) mod well_known_types;

pub mod fix_block_comments;
pub mod serialization_helpers;

pub use canonical_form::{
    CanonicalForm, CanonicalizeError, ConstraintViolation, Reason, ValidationError,
};

pub use codegen::{ApiModel, Error};

#[cfg(feature = "bytes")]
pub use well_known_types::Bytes;

#[cfg(feature = "axum-support")]
pub mod axum_compat;

/// Reexport crates used by generated code.
///
/// This makes it much easier to keep the types in sync between the generated code and your own types.
pub mod reexport {
    pub use accept_header;
    pub use async_trait;
    #[cfg(feature = "axum-support")]
    pub use axum;
    #[cfg(feature = "axum-support")]
    pub use axum_extra;
    #[cfg(feature = "integer-restrictions")]
    pub use bounded_integer;
    pub use derive_more;
    #[cfg(feature = "axum-support")]
    pub use headers;
    pub use heck;
    pub use http;
    #[cfg(feature = "api-problem")]
    pub use http_api_problem;
    pub use mime;
    #[cfg(feature = "string-pattern")]
    pub use regress;
    pub use serde;
    pub use serde_enum_str;
    pub use serde_json;
    pub use serde_with;
    pub use time;
    #[cfg(feature = "uuid")]
    pub use uuid;
}
