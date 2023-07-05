use openapiv3::{OpenAPI, ReferenceOr};

use crate::{
    codegen::{find_well_known_type, Scalar},
    resolve_trait::Resolve,
};

#[derive(Debug, Clone)]
pub(crate) enum OrScalar<T> {
    Item(T),
    Scalar(Scalar),
}

impl<T> From<Scalar> for OrScalar<T> {
    fn from(value: Scalar) -> Self {
        Self::Scalar(value)
    }
}

impl<T> OrScalar<T>
where
    ReferenceOr<T>: Resolve<Output = T>,
{
    /// Convert a `ReferenceOr<T>` into an `OrScalar<T>`.
    ///
    /// Well-known types are converted into the appropriate scalar.
    ///
    /// Unknown references, including those which look like internal references
    /// but are not found,  are converted into `Scalar::Any`.
    pub(crate) fn new<'a>(spec: &'a OpenAPI, ref_: &'a ReferenceOr<T>) -> OrScalar<&'a T> {
        if let Some(wkt) = ref_.as_ref_str().and_then(find_well_known_type) {
            return wkt.into();
        }
        match ref_.resolve(spec) {
            Ok(t) => OrScalar::Item(t),
            Err(_) => OrScalar::Scalar(Scalar::Any),
        }
    }
}
