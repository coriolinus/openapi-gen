use super::{
    api_model::{Ref, Reference, UnknownReference},
    Item,
};
use proc_macro2::TokenStream;
use quote::quote;

/// An inline definition of a mapping from String to T
#[derive(Debug, Clone)]
pub struct Map<Ref = Reference> {
    pub value_type: Option<Ref>,
}

impl Map<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Map<Reference>, UnknownReference> {
        let Self { value_type } = self;
        let value_type = value_type.map(|ref_| resolver(&ref_)).transpose()?;
        Ok(Map { value_type })
    }
}

// impl Map {
//     pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
//         let value_referent = self
//             .value_type
//             .as_ref()
//             .map(|vt| {
//                 let vt_ident = Item::reference_referent_ident(vt, derived_name);
//                 quote!(#vt_ident)
//             })
//             .unwrap_or(quote!(serde_json::Value));
//         quote!(std::collections::HashMap<String, #value_referent>)
//     }
// }
