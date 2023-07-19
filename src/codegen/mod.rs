//! Working directly with the OpenAPI structs gives us an OpenAPI-flavored view of the world.
//! This view does not map neatly to Rust. Instead, we want to construct our own object model which more neatly
//! maps to our output types. This module contains the definitions for that model.

pub(crate) mod api_model;
pub(crate) mod endpoint;
pub(crate) mod item;
pub(crate) mod rust_keywords;
pub(crate) mod value;
pub(crate) mod well_known_types;

pub(crate) use api_model::Ref;
pub use {
    api_model::{ApiModel, Error, Reference, UnknownReference},
    endpoint::Endpoint,
    item::Item,
    value::{
        list::List, map::Map, object::Object, one_of_enum::OneOfEnum, scalar::Scalar, set::Set,
        string_enum::StringEnum, Value, ValueConversionError,
    },
    well_known_types::find_well_known_type,
};

use proc_macro2::Span;
use syn::Ident;

/// We always want call-site semantics for our identifiers, so
/// they can be accessed from peer code.
pub fn make_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
