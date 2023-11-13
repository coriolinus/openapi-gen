use std::{cell::OnceCell, fmt};

use crate::{
    codegen::{
        api_model::{AsBackref, Ref, Reference, UnknownReference},
        make_ident,
    },
    ApiModel,
};

use heck::AsUpperCamelCase;
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use proc_macro2::TokenStream;
use quote::quote;

use super::ValueConversionError;

#[derive(Debug, Clone)]
pub struct Variant<Ref = Reference> {
    pub definition: Ref,
    pub mapping_name: Option<String>,
    pub status_code: Option<http::StatusCode>,
    computed_name: OnceCell<String>,
}

impl<R> Variant<R> {
    pub(crate) fn new(definition: R, mapping_name: Option<String>) -> Self {
        Self {
            definition,
            mapping_name,
            status_code: None,
            computed_name: OnceCell::new(),
        }
    }

    /// Get the computed name of this variant if it exists.
    ///
    /// The final computed name for this variant is determined at `emit_definition`,
    /// when the name resolver is available. In general, code which requires access to the computed
    /// name of the variant should only run after its definition has been emitted.
    // TODO: can we improve on that, compute it earlier somehow?
    //
    // This function is not currently called from all feature sets, so it might erroneously trigger
    // a dead code warning without this annotation.
    #[allow(dead_code)]
    pub(crate) fn computed_name(&self) -> Option<&str> {
        self.computed_name.get().map(String::as_str)
    }
}

impl Variant<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Variant<Reference>, UnknownReference> {
        let Self {
            definition,
            mapping_name,
            status_code,
            computed_name,
        } = self;
        let definition = resolver(&definition)?;
        Ok(Variant {
            definition,
            mapping_name,
            status_code,
            computed_name,
        })
    }
}

impl Variant {
    /// Compute an appropriate variant identifier for this variant.
    ///
    /// `idx` should be the index of this variant among all variants in the enum.
    ///
    /// Rules:
    ///
    /// - If there is an explicit mapping, use it
    /// - Else use the variant's identifier
    /// - Else use `Variant{idx:02}`.
    fn compute_variant_name<'a>(
        &self,
        idx: usize,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> &str {
        self.computed_name
            .get_or_init(|| {
                self.mapping_name
                    .as_deref()
                    .or_else(|| name_resolver(self.definition).ok())
                    .map(|name| format!("{}", AsUpperCamelCase(name)))
                    .unwrap_or_else(|| format!("Variant{idx:02}"))
            })
            .as_str()
    }

    pub(crate) fn serde_attributes(&self, default_name: &str) -> Vec<TokenStream> {
        let mut attributes = Vec::new();
        if let Some(mapping) = &self.mapping_name {
            if mapping != default_name {
                attributes.push(quote!(rename = #mapping));
            }
        }
        attributes
    }
}

/// OpenAPI's `oneOf` type
#[derive(Debug, Clone)]
pub struct OneOfEnum<Ref = Reference> {
    pub discriminant: Option<String>,
    pub variants: Vec<Variant<Ref>>,
}

impl<R> Default for OneOfEnum<R> {
    fn default() -> Self {
        Self {
            discriminant: Default::default(),
            variants: Default::default(),
        }
    }
}

impl<R> OneOfEnum<R> {
    pub(crate) fn use_serde_as_annotation(&self, model: &ApiModel<R>) -> bool
    where
        R: AsBackref + fmt::Debug,
    {
        self.variants.iter().any(|variant| {
            let Ok(item) = model.resolve(&variant.definition) else {
                return false;
            };
            item.serde_as_item_annotation(model).is_some()
        })
    }
}

impl OneOfEnum<Ref> {
    pub(crate) fn new(
        spec: &OpenAPI,
        model: &mut ApiModel<Ref>,
        spec_name: &str,
        rust_name: &str,
        schema: &Schema,
        variants: &[ReferenceOr<openapiv3::Schema>],
    ) -> Result<Self, ValueConversionError> {
        let schema_data = &schema.schema_data;

        let discriminant = schema_data
            .discriminator
            .as_ref()
            .map(|discriminant| discriminant.property_name.clone());

        let variants = variants
            .iter()
            .map(|schema_ref| {
                let mapping_name = schema_data
                    .discriminator
                    .as_ref()
                    .and_then(|discriminator| {
                        discriminator.mapping.iter().find_map(|(name, reference)| {
                            // it might seem incomplete to just pull out the `ref` variant of the
                            // `schema_ref` here, but that's actually per the docs:
                            //
                            // <https://docs.rs/openapiv3-extended/latest/openapiv3/struct.Discriminator.html>
                            //
                            // > When using the discriminator, inline schemas will not be considered.
                            (schema_ref.as_ref_str() == Some(reference.as_str()))
                                .then(|| name.to_owned())
                        })
                    });

                let definition = model
                    .convert_reference_or(
                        spec,
                        spec_name,
                        rust_name,
                        None,
                        &schema_ref.as_ref(),
                        None,
                    )
                    .map_err(ValueConversionError::from_inline(rust_name))?;

                Ok(Variant::new(definition, mapping_name))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            discriminant,
            variants,
        })
    }

    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<OneOfEnum<Reference>, UnknownReference> {
        let Self {
            discriminant,
            variants,
        } = self;
        let variants = variants
            .into_iter()
            .map(|variant| variant.resolve_refs(&resolver))
            .collect::<Result<_, _>>()?;
        Ok(OneOfEnum {
            discriminant,
            variants,
        })
    }
}

impl OneOfEnum {
    pub fn emit_definition<'a>(
        &self,
        model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let variants = self
            .variants
            .iter()
            .enumerate()
            .map(|(idx, variant)| {
                let variant_name = variant.compute_variant_name(idx, &name_resolver);
                let ident = make_ident(variant_name);
                let referent = model.definition(variant.definition, &name_resolver)?;
                let attributes = variant.serde_attributes(variant_name);
                let attributes =
                    (!attributes.is_empty()).then(|| quote!(#[serde( #( #attributes)* )]));
                Ok(quote! {
                    #attributes
                    #ident(#referent),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(quote! {
            {
                #( #variants )*
            }
        })
    }

    pub(crate) fn serde_container_attributes(&self) -> Vec<TokenStream> {
        let mut attributes = Vec::new();
        if let Some(tag) = &self.discriminant {
            attributes.push(quote!(tag = #tag));
        } else {
            attributes.push(quote!(untagged));
        }
        attributes
    }
}
