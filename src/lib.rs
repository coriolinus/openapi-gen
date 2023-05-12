mod canonical_form;
pub use canonical_form::{
    CanonicalForm, CanonicalizeError, ConstraintViolation, Reason, ValidationError,
};

mod well_known_types;
pub use well_known_types::Binary;
#[cfg(feature = "bytes")]
pub use well_known_types::Bytes;

/// Reexport crates used by generated code.
///
/// This makes it much easier to keep the types in sync between the generated code and your own types.
pub mod reexport {
    pub use async_trait;
    #[cfg(feature = "integer-restrictions")]
    pub use bounded_integer;
    pub use heck;
    pub use http_api_problem;
    #[cfg(feature = "string-pattern")]
    pub use regress;
    pub use serde;
    pub use time;
    #[cfg(feature = "uuid")]
    pub use uuid;
}
