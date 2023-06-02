use std::{collections::HashMap, ops::Deref};

use indexmap::IndexMap;
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use proc_macro2::TokenStream;
use quote::quote;

use crate::openapi_compat::{component_schemas, operation_inline_schemas, path_operations};

use super::{item::EmitError, rust_keywords::is_rust_keyword, Item, ParseItemError};

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

pub(crate) trait AsBackref {
    fn as_backref(&self) -> Option<usize>;
}

impl AsBackref for Reference {
    fn as_backref(&self) -> Option<usize> {
        Some(self.0)
    }
}

impl AsBackref for Ref {
    fn as_backref(&self) -> Option<usize> {
        match self {
            Ref::Back(idx) => Some(*idx),
            Ref::Forward(_) => None,
        }
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
}

impl<R> Default for ApiModel<R> {
    fn default() -> Self {
        Self {
            definitions: Default::default(),
            items: Default::default(),
            named_references: Default::default(),
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
}

// These functions only appear when we use potentially forward references.
impl ApiModel<Ref> {
    /// Compute a [`Ref`] from an OpenAPI [`ReferenceOr<Schema>`].
    ///
    /// In the event that this is an inline item definition, it is recursively added to the item set.
    /// Otherwise, the reference is simply converted to an appropriate `Ref`.
    pub fn convert_reference_or<S>(
        &mut self,
        name: &str,
        schema_ref: &ReferenceOr<S>,
    ) -> Result<Ref, Error>
    where
        S: Deref<Target = Schema>,
    {
        match schema_ref {
            ReferenceOr::Item(schema) => self.add_inline_items(name, None, schema),
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
        name: &str,
        reference_name: Option<&str>,
        schema: &Schema,
    ) -> Result<Ref, Error> {
        let item = Item::parse_schema(self, name, schema)?;

        // when constructing the item, we might have discovered that it needs a somewhat different name than planned.
        // extract that.
        let name = item.name.clone();

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

        Ok(ApiModel {
            definitions,
            items,
            named_references,
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

impl TryFrom<OpenAPI> for ApiModel {
    type Error = Error;

    fn try_from(spec: OpenAPI) -> Result<Self, Self::Error> {
        let mut models = ApiModel::<Ref>::default();

        // start by adding each schema defined in the components.
        // this gives the best chance of creating back refs instead of forward.
        for (name, schema) in component_schemas(&spec) {
            let reference_name = Some(format!("#/components/schemas/{name}"));
            models.add_inline_items(name, reference_name.as_deref(), schema)?;
        }

        for maybe_path_operation in path_operations(&spec) {
            let (path, operation_name, operation) =
                maybe_path_operation.map_err(Error::ResolvePathOperation)?;
            for (name, schema) in operation_inline_schemas(path, operation_name, operation) {
                models.add_inline_items(&name, None, schema)?;
            }
        }

        models.resolve_refs()
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
}
