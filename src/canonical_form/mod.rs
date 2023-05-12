mod impls;

#[derive(Debug, thiserror::Error, derive_more::From)]
pub enum Reason {
    #[error(transparent)]
    Err(Box<dyn std::error::Error>),
    #[error("{0}")]
    Reason(String),
}

impl<'a> From<&'a str> for Reason {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::Reason(value.into())
    }
}

impl Reason {
    #[inline]
    pub fn from_err<E: 'static + std::error::Error>(error: E) -> Self {
        Self::Err(Box::new(error))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("constraint violated: {0}")]
pub struct ConstraintViolation(Reason);

impl ConstraintViolation {
    #[inline]
    pub fn reason(reason: impl Into<Reason>) -> Self {
        Self(reason.into())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("could not validate {type_name} from an instance of its canonical form")]
pub struct ValidationError {
    type_name: &'static str,
    #[source]
    source: Reason,
}

impl ValidationError {
    #[inline]
    pub fn reason<T>(reason: impl Into<Reason>) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            source: reason.into(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("could not produce an instance of canonical form from an instance of {type_name}")]
pub struct CanonicalizeError {
    type_name: &'static str,
    #[source]
    source: Reason,
}

impl CanonicalizeError {
    #[inline]
    pub fn reason<T>(reason: impl Into<Reason>) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            source: reason.into(),
        }
    }
}

/// Types which have a canonical form when expressed as a JSON-compatible item
/// implement this trait.
///
/// This is also where restrictions such as `multipleOf` are checked.
///
/// For simplicity, this is a copying operation; this trait copies all the data
/// passed through it, instead of taking ownership.
pub trait CanonicalForm: Sized {
    /// What type this is expressed as in JSON.
    type JsonRepresentation: std::fmt::Display;

    /// Ensure that all constraints on this type are upheld.
    ///
    /// Example: this is where things such as `multipleOf` are checked.
    fn check_constraints(&self) -> Result<(), ConstraintViolation> {
        Ok(())
    }

    /// Validate a particular JSON item as an instance of the canonical form.
    ///
    /// This function should handle basic parsing. If there exist additional
    /// constraints on `Self`, they should be validated in `check_constraints`,
    /// which should be called from within this function.
    ///
    /// The input can be anything which, when borrowed, is the same as the JSON representation.
    fn validate(from: &Self::JsonRepresentation) -> Result<Self, ValidationError>;

    /// Emit the canonical form as a JSON-compatible item.
    ///
    /// This function should first call `check_constraints` to validate this item's value.
    /// It should then transform the value into the json-compatible form.
    fn canonicalize(&self) -> Result<Self::JsonRepresentation, CanonicalizeError>;
}

/// Like `#[derive(CanonicalForm)]`, but without the proc macro.
///
/// Limitations:
///
/// - only for tuple-style newtypes
/// - must have visibility of `.0` in this scope
/// - no constraints are checked
///
/// ## Example
///
/// ```rust
/// # use openapi_gen::newtype_derive_canonical_form;
/// struct MyI64(i64);
/// newtype_derive_canonical_form!(MyI64, i64);
/// ```
#[macro_export]
macro_rules! newtype_derive_canonical_form {
    ($outer:path, $inner:path) => {
        impl openapi_gen::CanonicalForm for $outer {
            type JsonRepresentation = <$outer as CanonicalForm>::JsonRepresentation;
            fn validate<BorrowedRepresentation>(
                from: &BorrowedRepresentation,
            ) -> Result<Self, ValidationError>
            where
                Self::JsonRepresentation: Borrow<BorrowedRepresentation>,
                BorrowedRepresentation: ToOwned<Owned = Self::JsonRepresentation>,
            {
                let inner = $inner::validate(from)?;
                Ok(Self(inner))
            }
            fn canonicalize(
                &self,
            ) -> Result<Self::JsonRepresentation, openapi_gen::CanonicalizeError> {
                self.0.canonicalize()
            }
        }
    };
}
