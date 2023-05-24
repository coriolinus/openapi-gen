use crate::codegen::make_ident;

use super::Item;
use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
use openapiv3::ReferenceOr;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Debug, Clone)]
pub struct ObjectMember {
    pub definition: Box<ReferenceOr<Item>>,
    pub required: bool,
    pub read_only: bool,
    pub write_only: bool,
}

impl ObjectMember {
    fn emit_definition(&self, parent_derived_name: &str, member_name: &str) -> TokenStream {
        let sub_derived_name = format!(
            "{}{}",
            AsUpperCamelCase(parent_derived_name),
            AsUpperCamelCase(member_name)
        );
        let snake_member_name = make_ident(&format!("{}", AsSnakeCase(member_name)));
        let item_ref = Item::reference_referent_ident(&self.definition, &sub_derived_name);

        let (option_head, option_tail) = if !self.required {
            (quote!(Option<), quote!(>))
        } else {
            Default::default()
        };

        quote! {
            #snake_member_name: #option_head #item_ref #option_tail,
        }
    }
}

/// An inline definition of an object
#[derive(Debug, Clone)]
pub struct Object {
    pub members: IndexMap<String, ObjectMember>,
}

impl Object {
    pub fn emit_definition(&self, derived_name: &str) -> TokenStream {
        let members = self
            .members
            .iter()
            .map(|(member_name, member)| member.emit_definition(derived_name, member_name));
        quote! {
            {
                #( #members )*
            }
        }
    }
}
