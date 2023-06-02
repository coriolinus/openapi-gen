use proc_macro2::Span;
use syn::Ident;

pub mod media_type;
pub mod operation;

mod item_tree;
pub use item_tree::ApiModel;

/// We always want call-site semantics for our identifiers, so
/// they can be accessed from peer code.
pub fn make_ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
