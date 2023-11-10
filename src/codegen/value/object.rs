use std::fmt;

use crate::codegen::{
    api_model::{AsBackref, Ref, Reference, UnknownReference},
    make_ident, ApiModel, PropertyOverride,
};

use heck::{AsSnakeCase, AsUpperCamelCase};
use indexmap::IndexMap;
use openapiv3::{ObjectType, OpenAPI};
use proc_macro2::TokenStream;
use quote::quote;

use super::ValueConversionError;

pub(crate) const BODY_IDENT: &str = "body";

#[derive(Debug, Clone)]
pub struct ObjectMember<Ref = Reference> {
    pub definition: Ref,
    pub read_only: bool,
    pub write_only: bool,
    pub inline_option: bool,
}

impl ObjectMember<Ref> {
    pub(crate) fn new(definition: Ref) -> Self {
        Self {
            definition,
            read_only: false,
            write_only: false,
            inline_option: false,
        }
    }

    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<ObjectMember<Reference>, UnknownReference> {
        let Self {
            definition,
            read_only,
            write_only,
            inline_option,
        } = self;
        let definition = resolver(&definition)?;
        Ok(ObjectMember {
            definition,
            read_only,
            write_only,
            inline_option,
        })
    }
}

impl ObjectMember {
    fn emit_definition<'a>(
        &self,
        member_name: &str,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let docs = model[self.definition]
            .docs
            .as_deref()
            .map(|docs| quote!(#[doc = #docs]));

        let item = model.resolve(self.definition).ok();
        let get_property_override = |override_value: &dyn Fn(&PropertyOverride) -> bool| -> bool {
            item.and_then(|item| item.value.as_property_override())
                .map(override_value)
                .unwrap_or_default()
        };
        let read_only = self.read_only || get_property_override(&|prop| prop.read_only);
        let write_only = self.write_only || get_property_override(&|prop| prop.write_only);

        let serde_as = item
            .and_then(|item| item.use_display_from_str(model))
            .map(|annotation| {
                let annotation = if self.inline_option {
                    quote!(Option<#annotation>)
                } else {
                    annotation
                };

                let annotation = annotation.to_string().replace(' ', "");
                quote!(#[serde_as(as = #annotation)])
            });

        let mut serde_attributes = Vec::new();

        if read_only {
            serde_attributes.push(quote!(skip_deserializing));
        }
        if write_only {
            serde_attributes.push(quote!(skip_serializing));
        }

        let mut snake_member_name = format!("{}", AsSnakeCase(member_name));
        model.deconflict_member_or_variant_ident(&mut snake_member_name);
        let snake_member_name = make_ident(&snake_member_name);
        let mut item_ref = model.definition(self.definition, name_resolver)?;

        if snake_member_name != member_name {
            serde_attributes.push(quote!(rename = #member_name));
        }

        // `self.inline_option` is set when this item is optional, not intrinsically,
        // but within the context of this object.
        if self.inline_option {
            item_ref = quote!(Option<#item_ref>);
            serde_attributes.push(quote!(skip_serializing_if = "Option::is_none"));
        }

        let serde_attributes = (!serde_attributes.is_empty()).then(|| {
            quote! {
                #[serde(#( #serde_attributes ),*)]
            }
        });

        Ok(quote! {
            #docs
            #serde_as
            #serde_attributes
            pub #snake_member_name: #item_ref,
        })
    }
}

/// An inline definition of an object
#[derive(Debug, Clone)]
pub struct Object<Ref = Reference> {
    pub members: IndexMap<String, ObjectMember<Ref>>,
    /// When `true`, this object is a generated response. It may or may not contain a body member. If it does,
    /// it is named `BODY_IDENT`. All other members are assumed to be headers.
    ///
    /// This gets used when generating framework-specific code, to transform the response enum into an appropriate
    /// `(status, headers, body)` tuple. It is necessary because when the code generator notices that there are no
    /// defined headers in a response, it uses the body object as the only return value of the appropriate trait method.
    ///
    /// This means that when this is `false`, we can just return that value as the body. When it is `true`, we must
    /// generate code to unpack the struct appropriately.
    pub is_generated_body_and_headers: bool,
}

impl<R> Default for Object<R> {
    fn default() -> Self {
        Self {
            members: Default::default(),
            is_generated_body_and_headers: Default::default(),
        }
    }
}

impl<R> Object<R> {
    pub(crate) fn use_serde_as_annotation(&self, model: &ApiModel<R>) -> bool
    where
        R: AsBackref + fmt::Debug,
    {
        self.members.values().any(|member| {
            let Ok(item) = model.resolve(&member.definition) else {
                return false;
            };
            item.use_display_from_str(model).is_some()
        })
    }
}

impl Object<Ref> {
    pub(crate) fn new(
        spec: &OpenAPI,
        model: &mut ApiModel<Ref>,
        _spec_name: &str,
        rust_name: &str,
        object_type: &ObjectType,
    ) -> Result<Self, ValueConversionError> {
        let members = object_type
            .properties
            .iter()
            .map::<Result<_, ValueConversionError>, _>(|(member_name, schema_ref)| {
                // TODO: should we resolve references here instead? But that kind of doesn't make sense.
                // For now, we're only permitting inline definitions to be read/write-only.
                let read_only = schema_ref
                    .as_item()
                    .map(|schema| schema.schema_data.read_only)
                    .unwrap_or_default();

                let write_only = schema_ref
                    .as_item()
                    .map(|schema| schema.schema_data.write_only)
                    .unwrap_or_default();

                // If a model exists for the bare property name, qualify
                // this one with the object name.
                //
                // This isn't perfect, because the first member we
                // encounter with a particular name gets to keep the
                // bare name, while others get their names qualified,
                // but it's still better than just appending digits.
                let ucc_member_name = format!("{}", AsUpperCamelCase(member_name));
                let mut rust_name = if model.ident_exists(&ucc_member_name) {
                    format!("{rust_name}{ucc_member_name}")
                } else {
                    ucc_member_name
                };
                model.deconflict_ident(&mut rust_name);

                // This is expected to be the only place where we construct a non-`None` instance of `containing_object`.
                let containing_object = Some((object_type, member_name.as_str()));

                let definition = model
                    .convert_reference_or(
                        spec,
                        member_name,
                        &rust_name,
                        None,
                        &schema_ref.as_ref(),
                        containing_object,
                    )
                    .map_err(ValueConversionError::from_inline(&rust_name))?;

                // In a perfect world, we could just set the item's `nullable` field here in the
                // event that we've determined that the item is in fact nullable.
                // Unfortunately, there are two obstacles to this.
                //
                // The first is relatively minor: the definition here might be a forward reference,
                // in which case we don't know that it is nullable in all contexts. We can work around that.
                //
                // The second is a bigger problem, though: making an item nullable the "proper" way
                // involves creating a type alias, which changes its public reference name. This doesn't
                // invalidate past references to the item--the structure of `ApiModel` avoids that issue--
                // but it does mean we'd need to design a single-purpose function `ApiModel::make_item_nullable`,
                // which just feels kind of ugly.
                //
                // Instead, we'll just handle nullable fields inline; that's good enough.
                let inline_option = !object_type.required.contains(member_name);

                Ok((
                    member_name.to_owned(),
                    ObjectMember {
                        definition,
                        read_only,
                        write_only,
                        inline_option,
                    },
                ))
            })
            .collect::<Result<_, _>>()?;
        Ok(Object {
            members,
            ..Default::default()
        })
    }

    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Object<Reference>, UnknownReference> {
        let Self {
            members,
            is_generated_body_and_headers,
        } = self;

        let members = members
            .into_iter()
            .map(|(name, member)| member.resolve_refs(&resolver).map(|member| (name, member)))
            .collect::<Result<_, _>>()?;

        Ok(Object {
            members,
            is_generated_body_and_headers,
        })
    }
}

impl Object {
    pub fn emit_definition<'a>(
        &self,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let members = self
            .members
            .iter()
            .map(|(member_name, member)| member.emit_definition(member_name, model, &name_resolver))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(quote! {
            {
                #( #members )*
            }
        })
    }
}
