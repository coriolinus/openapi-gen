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
use quote::quote;

mod context;
pub use context::Context;

fn maybe_map_reference_or<T, O, E>(
    reference: ReferenceOr<T>,
    map: impl FnOnce(T) -> Result<O, E>,
) -> Result<ReferenceOr<O>, E> {
    match reference {
        ReferenceOr::Reference { reference } => Ok(ReferenceOr::Reference { reference }),
        ReferenceOr::Item(t) => Ok(ReferenceOr::Item(map(t)?)),
    }
}

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

/// List of Rust Keywords.
///
/// These are prohibited as item names.
///
/// See <https://doc.rust-lang.org/reference/keywords.html>.
const RUST_KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "async",
    "await",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "final",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro_rules",
    "macro",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "pub",
    "ref",
    "return",
    "self",
    "Self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "union",
    "unsafe",
    "unsized",
    "use",
    "virtual",
    "where",
    "while",
    "yield",
];
