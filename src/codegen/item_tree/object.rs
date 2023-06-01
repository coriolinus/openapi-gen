use crate::codegen::make_ident;

use super::{
    api_model::{Ref, Reference, UnknownReference},
    Item,
};
use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct ObjectMember<Ref = Reference> {
    pub definition: Ref,
    pub required: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl ObjectMember<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<ObjectMember<Reference>, UnknownReference> {
        let Self {
            definition,
            required,
            read_only,
            write_only,
        } = self;
        let definition = resolver(&definition)?;
        Ok(ObjectMember {
            definition,
            required,
            read_only,
            write_only,
        })
    }
}

// impl ObjectMember {
//     fn emit_definition(&self, parent_derived_name: &str, member_name: &str) -> TokenStream {
//         let docs = self
//             .definition
//             .as_item()
//             .and_then(|item| item.docs.as_deref())
//             .map(|docs| quote!(#[doc = #docs]));

//         let sub_derived_name = format!(
//             "{}{}",
//             AsUpperCamelCase(parent_derived_name),
//             AsUpperCamelCase(member_name)
//         );
//         let snake_member_name = make_ident(&format!("{}", AsSnakeCase(member_name)));
//         let item_ref = Item::reference_referent_ident(&self.definition, &sub_derived_name);

//         let (option_head, option_tail) = if !self.required {
//             (quote!(Option<), quote!(>))
//         } else {
//             Default::default()
//         };

//         quote! {
//             #docs
//             #snake_member_name: #option_head #item_ref #option_tail,
//         }
//     }
// }

/// An inline definition of an object
#[derive(Debug, Clone)]
pub struct Object<Ref = Reference> {
    pub members: IndexMap<String, ObjectMember<Ref>>,
}

// impl Object {
//     pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
//         let members = self
//             .members
//             .iter()
//             .map(|(member_name, member)| member.emit_definition(derived_name, member_name));
//         quote! {
//             {
//                 #( #members )*
//             }
//         }
//     }
// }

impl Object<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Object<Reference>, UnknownReference> {
        let Self { members } = self;
        let members = members
            .into_iter()
            .map(|(name, member)| member.resolve_refs(&resolver).map(|member| (name, member)))
            .collect::<Result<_, _>>()?;
        Ok(Object { members })
    }
}
