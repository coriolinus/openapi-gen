//! Working directly with the OpenAPI structs gives us an OpenAPI-flavored view of the world.
//! This view does not map neatly to Rust. Instead, we want to construct our own object model which more neatly
//! maps to our output types. This module contains the definitions for that model.

pub(crate) mod rust_keywords;
pub mod well_known_types;

mod api_model;
pub use api_model::{ApiModel, Error};

mod item;
pub use item::{Item, ParseItemError};

mod list;
pub use list::List;

mod map;
pub use map::Map;

mod object;
pub use object::{Object, ObjectMember};

mod one_of_enum;
pub use one_of_enum::OneOfEnum;

mod scalar;
pub use scalar::Scalar;

mod set;
pub use set::Set;

mod string_enum;
pub use string_enum::StringEnum;

mod value;
pub use value::Value;
