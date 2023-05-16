use openapiv3::{OpenAPI, Schema};
use proc_macro2::TokenStream;

/// Make an item for this schema.
///
/// The generated item will be a `struct` or an `enum` according to the schema definition.
///
/// This will also produce an arbitrary number of subsidiary items, which were defined inline.
///
/// This produces only items defined inline. References are assumed to be defined elsewhere.
pub fn make_items_for_schema(spec: &OpenAPI, item_name: &str, schema: &Schema) -> TokenStream {
    todo!()
}
