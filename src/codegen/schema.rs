use openapiv3::{OpenAPI, Schema};
use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::item_tree::Item;

// / Make an item for this schema.
// /
// / The generated item will be a `struct` or an `enum` according to the schema definition.
// /
// / This will also produce an arbitrary number of subsidiary items, which were defined inline.
// /
// / This produces only items defined inline. References are assumed to be defined elsewhere.
// pub fn make_items_for_schema(_spec: &OpenAPI, item_name: &str, schema: &Schema) -> TokenStream {
//     let item = match Item::try_from(schema) {
//         Ok(item) => item,
//         Err(err) => {
//             let message = format!("failed to generate item from schema: {err}");
//             return quote!(compile_error!(#message));
//         }
//     };
//     item.emit(item_name)
// }
