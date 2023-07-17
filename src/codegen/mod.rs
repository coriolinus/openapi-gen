use proc_macro2::Span;
use syn::Ident;

mod item_tree;
pub(crate) use item_tree::{well_known_types::find_well_known_type, Scalar};
pub use item_tree::{ApiModel, Error};

/// We always want call-site semantics for our identifiers, so
/// they can be accessed from peer code.
pub fn make_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
