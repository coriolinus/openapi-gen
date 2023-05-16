//! Working directly with the OpenAPI structs gives us an OpenAPI-flavored view of the world.
//! This view does not map neatly to Rust. Instead, we want to construct our own object model which more neatly
//! maps to our output types. This module contains the definitions for that model.

mod scalar;
pub use scalar::Scalar;

mod list;
pub use list::List;

mod set;
pub use set::Set;

mod enum_;
pub use enum_::{OneOfEnum, StringEnum};

mod object;
pub use object::{Object, ObjectMember};

mod map;
pub use map::Map;

mod value;
pub use value::Value;

mod item;
pub use item::Item;

use openapiv3::ReferenceOr;

fn maybe_map_reference_or<T, O, E>(
    reference: ReferenceOr<T>,
    map: impl FnOnce(T) -> Result<O, E>,
) -> Result<ReferenceOr<O>, E> {
    match reference {
        ReferenceOr::Reference { reference } => Ok(ReferenceOr::Reference { reference }),
        ReferenceOr::Item(t) => Ok(ReferenceOr::Item(map(t)?)),
    }
}
