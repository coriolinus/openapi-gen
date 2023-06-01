use std::{borrow::Cow, collections::HashMap, ops::Deref};

use indexmap::IndexMap;
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use proc_macro2::{Ident, Span};

use crate::openapi_compat::{component_schemas, operation_inline_schemas, path_operations};

use super::{rust_keywords::is_rust_keyword, Item, ParseItemError};

/// We always want call-site semantics for our identifiers, so
/// they can be accessed from peer code.
fn make_ident_raw(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}

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

fn deconflict_keyword(ident_str: &str) -> Cow<str> {
    let mut ident_str = Cow::from(ident_str);
    if is_rust_keyword(&ident_str) {
        ident_str.to_mut().push('_');
    }
    ident_str
}

impl<R> ApiModel<R> {
    /// Make a struct member ident.
    ///
    /// Takes care of disambiguating from keywords.
    ///
    /// Does _not_ attempt to disambiguate from other members in the struct or
    /// variants in the enum.
    fn make_member_or_variant_ident(&self, ident_str: &str) -> Ident {
        let ident_str = deconflict_keyword(ident_str);
        make_ident_raw(&ident_str)
    }

    /// Make an identifier.
    ///
    /// Takes care of disambiguating from keywords and existing idents.
    ///
    /// Panics if `ident_str.is_empty()`.
    fn make_ident(&self, ident_str: &str) -> Ident {
        let ident_str = deconflict_keyword(ident_str);
        let mut ident = ident_str.to_string();
        let mut suffix = 1;
        while self.items.contains_key(&ident) {
            ident = format!("{ident_str}{suffix}");
            suffix += 1;
        }
        make_ident_raw(&ident)
    }

    /// Test whether an ident has already been created for a particular string.
    ///
    /// It is not an error to add new items with the same identifier; they'll be
    /// disambiguated. However, in certain cases such as struct properties, we might
    /// be able to come up with a better name than just "Foo2".
    pub fn ident_exists(&self, ident_str: &str) -> bool {
        let ident_str = deconflict_keyword(ident_str);
        let ident_str: &str = &ident_str;
        self.items.contains_key(ident_str)
    }

    /// Find the identifier associated with a particular reference.
    ///
    /// This computes in linear time over the number of items.
    pub(crate) fn find_name_for_reference(&self, ref_: &R) -> Option<&str>
    where
        R: AsBackref,
    {
        ref_.as_backref().and_then(|backref| {
            self.items
                .iter()
                .find_map(|(name, idx)| (*idx == backref).then_some(name.as_str()))
        })
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
        let idx = self.definitions.len();
        self.definitions.push(item);
        self.items.insert(name.to_owned(), idx);
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

impl std::ops::Index<Reference> for ApiModel<Reference> {
    type Output = Item;

    #[inline]
    fn index(&self, index: Reference) -> &Self::Output {
        &self.definitions[index.0]
    }
}

impl TryFrom<OpenAPI> for ApiModel<Reference> {
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
}
