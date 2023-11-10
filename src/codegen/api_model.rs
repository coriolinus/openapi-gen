use std::{
    borrow::Borrow,
    collections::HashMap,
    fmt,
    ops::Deref,
    path::{Path, PathBuf},
};

use heck::ToUpperCamelCase;
use indexmap::IndexMap;
use openapiv3::{OpenAPI, ReferenceOr, Schema};
use proc_macro2::TokenStream;
use quote::quote;

const OPENAPI_GEN_VERSION: &str = env!("CARGO_PKG_VERSION");
const OPENAPI_GEN_GIT_SHA: &str = env!("VERGEN_GIT_SHA");

#[cfg(feature = "axum-support")]
use crate::axum_compat;

use crate::{
    codegen::{
        endpoint::{
            self, insert_endpoints, parameter::insert_parameter, request_body::create_request_body,
            response::create_response_variants,
        },
        item::{EmitError, ParseItemError},
        make_ident,
        rust_keywords::is_rust_keyword,
        Endpoint, Item, Scalar,
    },
    fix_block_comments::fix_block_comments_to_string,
    openapi_compat::{
        component_headers, component_inline_and_external_schemas, component_parameters,
        component_requests, component_responses, OrScalar,
    },
};

use super::{
    endpoint::{
        header::{self, create_header},
        response::ResponseVariants,
    },
    item::ContainingObject,
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

pub trait AsBackref: Sized {
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
    /// Input file data
    input_file: Option<PathBuf>,
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
    /// Response variants by reference name.
    ///
    /// This is used to minimize duplication when merging variants from predefined response objects.
    ///
    /// Keys here are the qualified path to the reference name:
    /// `#/components/schemas/Foo`.
    pub(crate) response_variants: HashMap<String, ResponseVariants<Ref>>,
}

impl<R> Default for ApiModel<R> {
    fn default() -> Self {
        Self {
            input_file: Default::default(),
            definitions: Default::default(),
            items: Default::default(),
            named_references: Default::default(),
            endpoints: Default::default(),
            response_variants: Default::default(),
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
    pub(crate) fn get_named_reference(&self, reference: &str) -> Result<R, Error>
    where
        R: AsBackref,
    {
        self.named_references
            .get(reference)
            .copied()
            .map(AsBackref::from_backref)
            .ok_or_else(|| Error::UnknownReference(UnknownReference(reference.into())))
    }

    /// Insert an item definition for the last item in the items list.
    ///
    /// This ensures that item has a unique name.
    ///
    /// This errors if this would change an existing definition.
    ///
    /// This panics if the items list is empty.
    ///
    /// On success, returns the index to which the inserted name points.
    fn insert_item_name(&mut self) -> usize {
        let new = self
            .definitions
            .len()
            .checked_sub(1)
            .expect("definitions list is not empty");

        // we need to do a little ownership dance here so that we can deconflict
        // the name without borrow conflicts.
        // doing this shouldn't impose much if any cost; `take` is efficient.
        let mut name = std::mem::take(&mut self.definitions[new].rust_name);
        self.deconflict_ident(&mut name);
        self.definitions[new].rust_name = name.clone();

        if let Some(old) = self.items.insert(name, new) {
            // this branch is really never supposed to happen
            panic!("encountered a item name conflict despite deconflict_ident call: old {old}, new {new}, name: {}", &self.definitions[new].rust_name);
        }
        new
    }

    /// Insert a named reference definition for the last item in the items list.
    ///
    /// This errors if it would change an existing definition.
    ///
    /// This panics if the items list is empty.
    fn insert_item_named_reference(&mut self, named_reference: &str) -> Result<(), Error> {
        let new = self
            .definitions
            .len()
            .checked_sub(1)
            .expect("definitions list is not empty");
        if let Some(old) = self
            .named_references
            .insert(named_reference.to_owned(), new)
        {
            return Err(Error::DuplicateItemName {
                name: named_reference.to_owned(),
                old,
                new,
            });
        }
        Ok(())
    }

    /// Insert a new named reference definition for a given reference.
    ///
    /// Will error if `ref_` is not a back-reference.
    pub(crate) fn insert_named_reference_for(
        &mut self,
        named_reference: &str,
        ref_: &R,
    ) -> Result<(), Error>
    where
        R: AsBackref + std::fmt::Debug,
    {
        let ref_ = ref_
            .as_backref()
            .ok_or_else(|| UnknownReference(format!("{ref_:?}")))?;
        let previous = self
            .named_references
            .insert(named_reference.to_owned(), ref_);
        if let Some(old) = previous {
            if old != ref_ {
                return Err(Error::DuplicateItemName {
                    name: named_reference.to_string(),
                    old,
                    new: ref_,
                });
            }
        }
        Ok(())
    }

    /// Iterate references over all currently-known items.
    //
    // This function is not currently called from all feature sets, so it might erroneously trigger
    // a dead code warning without this annotation.
    #[allow(dead_code)]
    pub(crate) fn iter_items(&self) -> impl '_ + Iterator<Item = R>
    where
        R: AsBackref,
    {
        (0..self.items.len()).map(R::from_backref)
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
        spec: &OpenAPI,
        spec_name: &str,
        rust_name: &str,
        reference_name: Option<&str>,
        schema_ref: &ReferenceOr<S>,
        containing_object: ContainingObject,
    ) -> Result<Ref, Error>
    where
        S: Deref<Target = Schema>,
    {
        match schema_ref {
            ReferenceOr::Item(schema) => self.add_inline_items(
                spec,
                spec_name,
                rust_name,
                reference_name,
                schema,
                containing_object,
                None,
            ),
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
    //
    // Unfortunatley I think we're stuck with this argument count.
    #[allow(clippy::too_many_arguments)]
    pub fn add_inline_items(
        &mut self,
        spec: &OpenAPI,
        spec_name: &str,
        rust_name: &str,
        reference_name: Option<&str>,
        schema: &Schema,
        containing_object: ContainingObject,
        content_type: Option<String>,
    ) -> Result<Ref, Error> {
        let item = Item::parse_schema(
            spec,
            self,
            spec_name,
            rust_name,
            schema,
            containing_object,
            content_type,
        )?;
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
        self.definitions.push(item);
        let idx = self.insert_item_name();
        if let Some(reference_name) = reference_name {
            self.insert_item_named_reference(reference_name)?;
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
        let item = Item {
            spec_name: spec_name.to_owned(),
            rust_name: rust_name.to_owned(),
            value,
            ..Default::default()
        };
        self.definitions.push(item);
        let idx = self.insert_item_name();
        if let Some(reference_name) = reference_name {
            self.insert_item_named_reference(reference_name)?;
        }
        Ok(Ref::Back(idx))
    }

    /// Resolve all references into forward references, enabling the code generation use case.
    pub fn resolve_refs(self) -> Result<ApiModel<Reference>, Error> {
        let Self {
            input_file,
            definitions,
            items,
            named_references,
            endpoints,
            response_variants,
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

        let response_variants = response_variants
            .into_iter()
            .map(|(name, variants)| -> Result<_, UnknownReference> {
                let variants = variants
                    .into_iter()
                    .map(|variant| variant.resolve_refs(resolver))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((name, variants))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(ApiModel {
            input_file,
            definitions,
            items,
            named_references,
            endpoints,
            response_variants,
        })
    }
}

impl ApiModel {
    /// Emit the definition for an item by reference.
    ///
    /// If the item is trivial, emits the definition dirctly. Otherwise, emits the appropriate resolved name.
    pub(crate) fn definition<'a>(
        &self,
        ref_: Reference,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let item = self.resolve(ref_)?;
        match item.trivial_definition(self)? {
            Some(definition) => Ok(definition),
            None => {
                let name = name_resolver(ref_)?;
                let ident = make_ident(name);
                Ok(quote!(#ident))
            }
        }
    }

    /// Emit a footer to the module header with data about the input file
    fn emit_header_footer(&self) -> Option<String> {
        use md5::{Digest, Md5};

        let input_file = self.input_file.as_ref()?;
        let display = input_file.display();
        let data = std::fs::read(input_file).ok()?;
        let mut hasher = Md5::new();
        hasher.update(data);
        // this is terribly inefficient, but small enough that it doesn't matter
        let md5_hash = hasher
            .finalize()
            .into_iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<Vec<_>>()
            .join("");

        let out = format!(
            r#"
- input file: {display}
- input file md5 hash: {md5_hash}
        "#
        );
        Some(out.trim().to_owned())
    }

    /// Emit the module header.
    ///
    /// One can choose to skip omitting the module documentation. This is most useful in a testing context.
    fn emit_header(&self, emit_docs: bool) -> TokenStream {
        let mut out = quote!(#![allow(non_camel_case_types)]);
        if emit_docs {
            let timestamp = time::OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .expect("can always form a RFC3339 from current utc");

            let mut header_doc = format!(
                r#"
This module was automatically generated by [`openapi-gen`] from an OpenAPI source file.

Your changes may be overwritten without notice.

[`openapi-gen`]: https://github.com/coriolinus/openapi-gen/

## Run Data

- timestamp: {timestamp}
- `openapi-gen` version: {OPENAPI_GEN_VERSION}
- `openapi-gen` git sha: {OPENAPI_GEN_GIT_SHA}
"#
            );
            if let Some(footer) = self.emit_header_footer() {
                use std::fmt::Write;

                let _ = writeln!(&mut header_doc, "{footer}");
            }
            let header_doc = header_doc.trim();

            out = quote! {
                #![doc = #header_doc]
                #out
            };
        }
        out
    }

    /// Emit the items of this model as a token stream.
    ///
    /// This is largely for future-proofing, so we can embed this more easily in
    /// a proc macro in the future if we so desire.
    pub fn emit_items_to_token_stream(&self, emit_docs: bool) -> Result<TokenStream, Error> {
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

        let header = self.emit_header(emit_docs);

        let items = self
            .definitions
            .iter()
            .map(|item| item.emit(self, &name_resolver))
            .collect::<Result<Vec<_>, _>>()?;

        let endpoints = self
            .endpoints
            .iter()
            .map(|endpoint| endpoint.emit(self, &name_resolver))
            .collect::<Result<Vec<_>, _>>()?;

        let trait_api = quote! {
            #[openapi_gen::reexport::async_trait::async_trait]
            pub trait Api {
                #(
                    #endpoints
                )*
            }
        };

        #[cfg(not(feature = "axum-support"))]
        let axum = TokenStream::default();
        #[cfg(feature = "axum-support")]
        let axum = axum_compat::axum_items(self, &name_resolver)?;

        Ok(quote! {
            #header
            #( #items )*
            #trait_api
            #axum
        })
    }

    /// Emit the items defined by this model as Rust code.
    pub fn emit_items(&self, emit_docs: bool) -> Result<String, Error> {
        let tokens = self.emit_items_to_token_stream(emit_docs)?;
        let buffer = tokens.to_string();
        let file = syn::parse_str::<syn::File>(&buffer)
            .map_err(|err| Error::CodegenParse { err, buffer })?;
        let pretty = prettyplease::unparse(&file);
        let fixed =
            fix_block_comments_to_string(&pretty).map_err(Error::fix_block_comments(&pretty))?;
        Ok(fixed)
    }
}

impl std::ops::Index<Reference> for ApiModel<Reference> {
    type Output = Item;

    #[inline]
    fn index(&self, index: Reference) -> &Self::Output {
        &self.definitions[index.0]
    }
}

impl<R> ApiModel<R>
where
    R: AsBackref + fmt::Debug,
{
    #[inline]
    pub fn resolve(&self, ref_: impl Borrow<R>) -> Result<&Item<R>, UnknownReference> {
        let ref_ = ref_.borrow();
        let unknown_reference = || UnknownReference(format!("{ref_:?}"));
        let idx = ref_.as_backref().ok_or_else(unknown_reference)?;
        self.definitions.get(idx).ok_or_else(unknown_reference)
    }

    #[inline]
    pub fn resolve_mut(&mut self, ref_: impl Borrow<R>) -> Result<&mut Item<R>, UnknownReference> {
        let ref_ = ref_.borrow();
        let unknown_reference = || UnknownReference(format!("{ref_:?}"));
        let idx = ref_.as_backref().ok_or_else(unknown_reference)?;
        self.definitions.get_mut(idx).ok_or_else(unknown_reference)
    }
}

impl ApiModel {
    pub fn new(spec: &OpenAPI, input_file: Option<impl AsRef<Path>>) -> Result<Self, Error> {
        let input_file = input_file.map(|path_ref| path_ref.as_ref().to_owned());
        let mut model = ApiModel::<Ref> {
            input_file,
            ..Default::default()
        };

        // component schemas
        for (spec_name, reference_name, schema_or_scalar) in
            component_inline_and_external_schemas(spec)
        {
            let rust_name = spec_name.to_upper_camel_case();
            let reference_name = Some(reference_name);
            let reference_name = reference_name.as_deref();
            let ref_ = match schema_or_scalar {
                OrScalar::Item(schema) => model.add_inline_items(
                    spec,
                    spec_name,
                    &rust_name,
                    reference_name,
                    schema,
                    None,
                    None,
                )?,
                OrScalar::Scalar(scalar) => {
                    model.add_scalar(spec_name, &rust_name, reference_name, scalar)?
                }
            };
            // all top-level components are public, even if they are typedefs
            if let Ok(item) = model.resolve_mut(&ref_) {
                item.pub_typedef = true;
            }
        }

        // component parameters
        for (_spec_name, reference_name, param) in component_parameters(spec) {
            let reference_name = Some(reference_name);
            let reference_name = reference_name.as_deref();
            let ref_ = insert_parameter(spec, &mut model, reference_name, param)
                .map_err(Error::InsertComponentParameter)?;
            // all top-level component parameters are also public
            if let Ok(item) = model.resolve_mut(&ref_) {
                item.pub_typedef = true;
            }
        }

        // component requests
        for (spec_name, reference_name, request_body) in component_requests(spec) {
            let reference_name = Some(reference_name);
            let reference_name = reference_name.as_deref();
            let ref_ =
                create_request_body(spec, &mut model, spec_name, reference_name, request_body)?;
            // all top-level component parameters are also public
            if let Ok(item) = model.resolve_mut(&ref_) {
                item.pub_typedef = true;
                item.nullable = !request_body.required;
                if item.docs.is_none() {
                    item.docs = request_body.description.clone();
                }
            }
        }

        // component responses
        for (spec_name, reference_name, response) in component_responses(spec) {
            let variants = create_response_variants(spec, &mut model, spec_name, None, response)?;
            model.response_variants.insert(reference_name, variants);
        }

        // component headers
        for (spec_name, reference_name, header) in component_headers(spec) {
            let reference_name = Some(reference_name);
            let reference_name = reference_name.as_deref();
            let ref_ = create_header(spec, &mut model, spec_name, reference_name, header)?;
            if let Ok(item) = model.resolve_mut(&ref_) {
                item.impl_header = true;
                // all top-level component parameters are also public
                item.pub_typedef = true;
            }
        }

        insert_endpoints(spec, &mut model)?;

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
    ParseEndpoint(#[from] endpoint::Error),
    #[error("inserting component parameter")]
    InsertComponentParameter(#[source] anyhow::Error),
    #[error("duplicate item name: {name} (old: {old}, new: {new})")]
    DuplicateItemName {
        name: String,
        old: usize,
        new: usize,
    },
    #[error("inserting component headers")]
    InsertHeader(#[from] header::Error),
    #[cfg(feature = "axum-support")]
    #[error("implementing axum compatibility")]
    AxumCompat(#[from] axum_compat::Error),
    #[error("fixing block comments")]
    FixBlockComments {
        #[source]
        err: std::io::Error,
        pretty: String,
    },
}

impl Error {
    fn fix_block_comments(pretty: &str) -> impl '_ + FnOnce(std::io::Error) -> Self {
        move |err| {
            let pretty = pretty.to_owned();
            Self::FixBlockComments { err, pretty }
        }
    }
}
