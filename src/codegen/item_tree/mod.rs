//! Working directly with the OpenAPI structs gives us an OpenAPI-flavored view of the world.
//! This view does not map neatly to Rust. Instead, we want to construct our own object model which more neatly
//! maps to our output types. This module contains the definitions for that model.

mod scalar;
use proc_macro2::TokenStream;
pub use scalar::Scalar;

mod list;
pub use list::List;

mod set;
pub use set::Set;

mod one_of_enum;
pub use one_of_enum::OneOfEnum;

mod string_enum;
pub use string_enum::StringEnum;

mod object;
pub use object::{Object, ObjectMember};

mod map;
pub use map::Map;

mod value;
pub use value::Value;

mod item;
pub use item::{Item, ParseItemError};

use quote::quote;

mod api_model;
pub use api_model::{ApiModel, Error};

pub(crate) mod rust_keywords;
pub mod well_known_types;

fn default_derives() -> Vec<TokenStream> {
    vec![
        quote!(Debug),
        quote!(Clone),
        quote!(PartialEq),
        quote!(openapi_gen::reexport::serde::Serialize),
        quote!(openapi_gen::reexport::serde::Deserialize),
        quote!(openapi_gen::reexport::derive_more::Constructor),
    ]
}
