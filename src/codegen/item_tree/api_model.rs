use std::{collections::HashMap, ops::Deref};

use heck::ToUpperCamelCase;
use indexmap::IndexMap;
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use proc_macro2::TokenStream;
use quote::quote;

use crate::openapi_compat::{
    component_inline_and_external_schemas, component_parameters, OrScalar,
};

use super::{
    endpoint::{insert_endpoints, Endpoint},
    item::EmitError,
    rust_keywords::is_rust_keyword,
    Item, ParseItemError, Scalar,
};

/// A reference to an item definition.
///
/// This can be dereferenced by an [`ApiModel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Reference(usize);

/// A Ref is either a back reference, or a forward reference.
///
/// This is an internal data structure which should never be exposed outside this crate.
/// All refs should get resolved into back references during the parsing process.
#[derive(Debug, Clone)]
pub(crate) enum Ref {
    Back(usize),
    Forward(String),
}

pub(crate) trait AsBackref: Sized {
    fn as_backref(&self) -> Option<usize>;
    fn from_backref(backref: usize) -> Self;
}

impl AsBackref for Reference {
    fn as_backref(&self) -> Option<usize> {
        Some(self.0)
    }

    fn from_backref(backref: usize) -> Self {
        Self(backref)
    }
}

impl AsBackref for Ref {
    fn as_backref(&self) -> Option<usize> {
        match self {
            Ref::Back(idx) => Some(*idx),
            Ref::Forward(_) => None,
        }
    }

    fn from_backref(backref: usize) -> Self {
        Self::Back(backref)
    }
}

#[derive(Debug, Clone)]
pub struct ApiModel<Ref = Reference> {
    /// Item definitions.
    definitions: Vec<Item<Ref>>,
    /// Map from the outer item identifier to the item.
    ///
    /// Keys here are just the identifier name: `Foo`.
    items: IndexMap<String, usize>,
    /// Map from named references to the item.
    ///
    /// Keys here are the qualified path to the reference name:
    /// `#/components/schemas/Foo`.
    named_references: HashMap<String, usize>,
    /// Api endpoints. These will be used later to generate `trait Api`.
    pub(crate) endpoints: Vec<Endpoint<Ref>>,
}

impl<R> Default for ApiModel<R> {
    fn default() -> Self {
        Self {
            definitions: Default::default(),
            items: Default::default(),
            named_references: Default::default(),
            endpoints: Default::default(),
        }
    }
}

impl<R> ApiModel<R> {
    /// Ensure a proposed ident does not conflict with Rust keywords.
    ///
    /// This does **not** deconflict with other idents, so should be used only for struct members
    /// or enum variants. For module-level idents, always prefer [`Self::deconflict_ident`].
    pub fn deconflict_member_or_variant_ident(&self, ident: &mut String) {
        if is_rust_keyword(ident) {
            ident.push('_');
        }
    }

    /// Ensure a proposed ident does not conflict with Rust keywords or other existing idents,
    /// by appending symbols until it is unique.
    pub fn deconflict_ident(&self, ident: &mut String) {
        self.deconflict_member_or_variant_ident(ident);
        let mut proposed_ident = ident.to_string();
        let mut suffix = 1;
        while self.items.contains_key(&proposed_ident) {
            proposed_ident = format!("{ident}{suffix}");
            suffix += 1;
        }

        *ident = proposed_ident
    }

    /// Test whether an ident has already been created for a particular string.
    ///
    /// It is not an error to add new items with the same identifier; they'll be
    /// disambiguated. However, in certain cases such as struct properties, we might
    /// be able to come up with a better name than just "Foo2".
    pub fn ident_exists(&self, ident_str: &str) -> bool {
        self.items.contains_key(ident_str)
    }

    /// Get a reference from a named reference (`#/components/schemas/Foo`) if one exists among the definitions.
    pub(crate) fn get_named_reference(&self, reference: &str) -> Option<R>
    where
        R: AsBackref,
    {
        self.named_references
            .get(reference)
            .copied()
            .map(AsBackref::from_backref)
    }
}

// These functions only appear when we use potentially forward references.
impl ApiModel<Ref> {
    /// Compute a [`Ref`] from an OpenAPI [`ReferenceOr<Schema>`].
    ///
    /// In the event that this is an inline item definition, it is recursively added to the item set.
    /// Otherwise, the reference is simply converted to an appropriate `Ref`.
    pub fn convert_reference_or<S>(
        &mut self,
        spec_name: &str,
        rust_name: &str,
        schema_ref: &ReferenceOr<S>,
    ) -> Result<Ref, Error>
    where
        S: Deref<Target = Schema>,
    {
        match schema_ref {
            ReferenceOr::Item(schema) => self.add_inline_items(spec_name, rust_name, None, schema),
            ReferenceOr::Reference { reference } => match self.named_references.get(reference) {
                Some(position) => Ok(Ref::Back(*position)),
                None => Ok(Ref::Forward(reference.to_owned())),
            },
        }
    }

    /// Recursively add items to this model from a schema.
    ///
    /// All inline item definitions are added in topographic order.
    ///
    /// External item definitions are permitted to be forward references.
    pub fn add_inline_items(
        &mut self,
        spec_name: &str,
        rust_name: &str,
        reference_name: Option<&str>,
        schema: &Schema,
    ) -> Result<Ref, Error> {
        let item = Item::parse_schema(self, spec_name, rust_name, schema)?;
        self.add_item(item, reference_name)
    }

    /// Add a computed item to this model.
    ///
    /// Typically, `add_inline_items` will be more useful, but occasionally there is a reason
    /// to compute the item externally and add it here in a separate step.
    pub fn add_item(
        &mut self,
        item: Item<Ref>,
        reference_name: Option<&str>,
    ) -> Result<Ref, Error> {
        let name = item.rust_name.clone();

        let idx = self.definitions.len();
        self.definitions.push(item);
        self.items.insert(name, idx);
        if let Some(reference_name) = reference_name {
            self.named_references.insert(reference_name.to_owned(), idx);
        }

        Ok(Ref::Back(idx))
    }

    /// Add a typedef to a scalar to this model.
    pub fn add_scalar(
        &mut self,
        spec_name: &str,
        rust_name: &str,
        reference_name: Option<&str>,
        scalar: Scalar,
    ) -> Result<Ref, Error> {
        let value = scalar.into();
        let name = rust_name.to_owned();
        let item = Item {
            spec_name: spec_name.to_owned(),
            rust_name: rust_name.to_owned(),
            value,
            ..Default::default()
        };
        let idx = self.definitions.len();
        self.definitions.push(item);
        self.items.insert(name, idx);
        if let Some(reference_name) = reference_name {
            self.named_references.insert(reference_name.to_owned(), idx);
        }
        Ok(Ref::Back(idx))
    }

    /// Resolve all references into forward references, enabling the code generation use case.
    pub fn resolve_refs(self) -> Result<ApiModel<Reference>, Error> {
        let Self {
            definitions,
            items,
            named_references,
            endpoints,
        } = self;

        let resolver = |ref_: &Ref| match ref_ {
            Ref::Back(back) => Ok(Reference(*back)),
            Ref::Forward(fwd) => named_references
                .get(fwd)
                .copied()
                .map(Reference)
                .ok_or_else(|| UnknownReference(fwd.clone())),
        };

        let definitions = definitions
            .into_iter()
            .map(|item| item.resolve_refs(resolver))
            .collect::<Result<_, _>>()?;

        let endpoints = endpoints
            .into_iter()
            .map(|endpoint| endpoint.resolve_refs(resolver))
            .collect::<Result<_, _>>()?;

        Ok(ApiModel {
            definitions,
            items,
            named_references,
            endpoints,
        })
    }
}

impl ApiModel {
    /// Emit the items of this model as a token stream.
    ///
    /// This is largely for future-proofing, so we can embed this more easily in
    /// a proc macro in the future if we so desire.
    pub fn emit_items_to_token_stream(&self) -> Result<TokenStream, Error> {
        let names = self
            .items
            .iter()
            .map(|(name, &idx)| (idx, name.as_str()))
            .collect::<HashMap<_, _>>();
        let name_resolver = move |ref_: Reference| {
            names
                .get(&ref_.0)
                .copied()
                .ok_or_else(|| UnknownReference(format!("{ref_:?}")))
        };

        let items = self
            .definitions
            .iter()
            .map(|item| item.emit(self, &name_resolver))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(quote! {
            #( #items )*
        })
    }

    /// Emit the items defined by this model as Rust code.
    pub fn emit_items(&self) -> Result<String, Error> {
        let tokens = self.emit_items_to_token_stream()?;
        let buffer = tokens.to_string();
        let file = syn::parse_str::<syn::File>(&buffer)
            .map_err(|err| Error::CodegenParse { err, buffer })?;
        Ok(prettyplease::unparse(&file))
    }
}

impl std::ops::Index<Reference> for ApiModel<Reference> {
    type Output = Item;

    #[inline]
    fn index(&self, index: Reference) -> &Self::Output {
        &self.definitions[index.0]
    }
}

impl ApiModel<Reference> {
    #[inline]
    pub fn resolve(&self, ref_: Reference) -> Option<&Item> {
        self.definitions.get(ref_.0)
    }

    #[inline]
    pub fn resolve_mut(&mut self, ref_: Reference) -> Option<&mut Item> {
        self.definitions.get_mut(ref_.0)
    }
}

impl ApiModel<Ref> {
    #[inline]
    pub fn resolve(&self, ref_: &Ref) -> Option<&Item<Ref>> {
        match ref_ {
            Ref::Back(idx) => self.definitions.get(*idx),
            Ref::Forward(_) => None,
        }
    }

    #[inline]
    pub fn resolve_mut(&mut self, ref_: &Ref) -> Option<&mut Item<Ref>> {
        match ref_ {
            Ref::Back(idx) => self.definitions.get_mut(*idx),
            Ref::Forward(_) => None,
        }
    }
}

impl TryFrom<OpenAPI> for ApiModel {
    type Error = Error;

    fn try_from(spec: OpenAPI) -> Result<Self, Self::Error> {
        let mut model = ApiModel::<Ref>::default();

        // start by adding each schema defined in the components.
        // this gives the best chance of creating back refs instead of forward.
        for (spec_name, schema_or_scalar) in component_inline_and_external_schemas(&spec) {
            let rust_name = spec_name.to_upper_camel_case();
            let reference_name = Some(format!("#/components/schemas/{spec_name}"));
            let ref_ = match schema_or_scalar {
                OrScalar::Item(schema) => model.add_inline_items(
                    spec_name,
                    &rust_name,
                    reference_name.as_deref(),
                    schema,
                )?,
                OrScalar::Scalar(scalar) => {
                    model.add_scalar(spec_name, &rust_name, reference_name.as_deref(), scalar)?
                }
            };
            // all top-level components are public, even if they are typedefs
            if let Some(item) = model.resolve_mut(&ref_) {
                item.pub_typedef = true;
            }
        }

        // likewise for component parameters
        for (spec_name, param) in component_parameters(&spec) {
            let rust_name = spec_name.to_upper_camel_case();
            let reference_name = Some(format!("#/components/parameters/{spec_name}"));
            let Some(schema) = param.parameter_data_ref().schema().map(|schema_ref| schema_ref.resolve(&spec)) else { continue };
            let ref_ =
                model.add_inline_items(spec_name, &rust_name, reference_name.as_deref(), schema)?;
            // all top-level component parameters are also public
            if let Some(item) = model.resolve_mut(&ref_) {
                item.pub_typedef = true;
            }
        }

        insert_endpoints(&spec, &mut model)?;

        model.resolve_refs()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unknown reference: {0}")]
pub struct UnknownReference(pub String);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    UnknownReference(#[from] UnknownReference),
    #[error("parsing item schema")]
    ParseItem(#[from] ParseItemError),
    #[error("resolving path operation")]
    ResolvePathOperation(#[source] anyhow::Error),
    #[error("generating code")]
    Codegen(#[from] EmitError),
    #[error("generated code cannot be parsed as Rust")]
    CodegenParse {
        #[source]
        err: syn::parse::Error,
        buffer: String,
    },
    #[error("parsing endpoint definition")]
    ParseEndpoint(#[from] super::endpoint::Error),
}
