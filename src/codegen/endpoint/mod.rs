use std::fmt;

use heck::{AsUpperCamelCase, ToSnakeCase, ToUpperCamelCase};
use indexmap::IndexMap;
use openapiv3::OpenAPI;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    codegen::{
        endpoint::parameter::ParameterLocation, make_ident, Item, Object, Ref, Reference,
        UnknownReference,
    },
    openapi_compat::path_items,
    ApiModel,
};

pub(crate) mod header;

pub(crate) mod parameter;
use parameter::{convert_param_ref, Parameter};

pub(crate) mod request_body;

pub(crate) mod response;

pub(crate) mod verb;
use verb::Verb;

use super::{api_model::AsBackref, value::object::ObjectMember};

#[derive(Debug, Clone)]
pub struct Endpoint<Ref = Reference> {
    /// Relative path from the server URL to this endpoint.
    ///
    /// May contain parameters with `{name}` notation.
    pub path: String,
    /// Endpoint-level documentation.
    ///
    /// Preferentially taken from `PathItem::description`, falling back to `PathItem::summary` if required.
    pub endpoint_documentation: Option<String>,
    /// Operation-level endpoint documentation.
    ///
    /// If external documentation is specified, that is queried first. Otherwise, this falls back first to
    /// `Operation::description` and then to `Operation::summary`.
    pub operation_documentation: Option<String>,
    pub verb: Verb,
    /// Query parameters are grouped together into a single object, which has one field per parameter.
    ///
    /// This is useful because Axum supports only one `Query` extractor per request, so query parameters must be coalesced.
    ///
    /// They are derived first from `PathItem::parameters`, then updated from `Operation::parameters`.
    pub query_parameters: Option<Ref>,
    /// Path parameters are grouped together into a single object, which has one field per parameter.
    ///
    /// This is useful because Axum supports only on `Path` extractor per request, so path parameters must be coalesced.
    ///
    /// They are derived first from `PathItem::parameters`, then updated from `Operation::parameters`.
    pub path_parameters: Option<Ref>,
    /// Headers by name.
    ///
    /// They are derived first from `PathItem::parameters`, then updated from `Operation::parameters`.`
    pub headers: IndexMap<String, Parameter<Ref>>,
    /// Operation ID.
    ///
    /// If set, this is used as the basis for the rust name.
    pub operation_id: Option<String>,
    /// Request body.
    ///
    /// This is overridden to `None` if `verb` is `GET`, `HEAD`, `DELETE`, or `TRACE`.
    pub request_body: Option<Ref>,
    /// Response body enum.
    ///
    /// This is always an enum, even in the event that there are 0 variants. (That is a degenerate case
    /// indicating a malformed OpenAPI specification).
    pub response: Ref,
}

type MaybeItemObject<'a, R> = Option<(R, &'a Item<R>, Object<R>)>;

impl<R> Endpoint<R> {
    /// Compute a documentation string for this endpoint.
    fn doc_string(&self) -> String {
        let internal_docs = match (
            self.endpoint_documentation.as_deref(),
            self.operation_documentation.as_deref(),
        ) {
            // we put operation docs before endpoint docs because they should in theory be more specific
            (Some(endpoint), Some(operation)) => Some(format!("{operation}\n\n{endpoint}\n\n")),
            (Some(docs), None) | (None, Some(docs)) => Some(docs.to_owned()),
            (None, None) => None,
        };

        let mut docs = internal_docs
            .map(|internal| format!("{internal}\n\n## Endpoint Data\n\n"))
            .unwrap_or_default();
        docs.push_str(&format!("`{} {}`\n\n", self.verb, self.path));
        if let Some(operation_id) = self.operation_id.as_deref() {
            docs.push_str("Operation ID: `");
            docs.push_str(operation_id);
            docs.push_str("`\n\n");
        }

        docs
    }

    /// Compute the function name for this endpoint.
    ///
    /// The endpoint doesn't know internally whether it is unique at the verb/path combo,
    /// or whether it needs a suffix to disambiguate from other content types. That must
    /// therefore be provided externally.
    pub(crate) fn function_name(&self, suffix: Option<&str>) -> String {
        compute_uncased_item_name(self.operation_id.as_deref(), self.verb, &self.path, suffix)
            .to_snake_case()
    }

    pub(crate) fn path_parameter_object<'a>(
        &'a self,
        model: &'a ApiModel<R>,
    ) -> Result<MaybeItemObject<'a, R>, UnknownReference>
    where
        R: 'static + AsBackref + fmt::Debug + Clone,
    {
        let obj = self
            .path_parameters
            .as_ref()
            .map(|ref_| model.resolve(ref_).map(|item| (ref_.clone(), item)))
            .transpose()?
            .map(|(ref_, item)| {
                (
                    ref_,
                    item,
                    item.value
                        .clone()
                        .try_into()
                        .expect("we only ever construct path_parameters as an object"),
                )
            });
        Ok(obj)
    }

    pub(crate) fn query_parameter_object<'a>(
        &'a self,
        model: &'a ApiModel<R>,
    ) -> Result<MaybeItemObject<'a, R>, UnknownReference>
    where
        R: 'static + AsBackref + fmt::Debug + Clone,
    {
        let obj = self
            .query_parameters
            .as_ref()
            .map(|ref_| model.resolve(ref_).map(|item| (ref_.clone(), item)))
            .transpose()?
            .map(|(ref_, item)| {
                (
                    ref_,
                    item,
                    item.value
                        .clone()
                        .try_into()
                        .expect("we only ever construct query_parameters as an object"),
                )
            });
        Ok(obj)
    }

    /// Compute the function parameters.
    ///
    /// Items are `(name, type, required)` where `name` is an appropriate parameter name, and `type` is convertable into a type ident.
    /// `required` is `true` when the item is mandatory.
    fn function_parameters<'a>(
        &'a self,
        model: &'a ApiModel<R>,
    ) -> Result<impl 'a + Iterator<Item = (String, R, bool)>, UnknownReference>
    where
        R: 'static + AsBackref + fmt::Debug + Clone,
    {
        // we emit function parameters in approximate order of predecence:
        // first path parameters,
        // then query parameters,
        // then headers

        let path_parameters = self.path_parameter_object(model)?.into_iter().flat_map(
            |(_ref_, _item, parameter_object)| {
                parameter_object.members.into_iter().map(|(name, member)| {
                    let name = name.to_snake_case();
                    let ref_ = member.definition;
                    let required = !member.inline_option;
                    (name, ref_, required)
                })
            },
        );

        let query_parameters = self.query_parameter_object(model)?.into_iter().flat_map(
            |(_ref_, _item, parameter_object)| {
                parameter_object.members.into_iter().map(|(name, member)| {
                    let name = name.to_snake_case();
                    let ref_ = member.definition;
                    let required = !member.inline_option;
                    (name, ref_, required)
                })
            },
        );

        let header_parameters = self.headers.iter().map(
            |(
                name,
                Parameter {
                    required, item_ref, ..
                },
            )| (name.to_snake_case(), item_ref.clone(), *required),
        );

        Ok(path_parameters
            .chain(query_parameters)
            .chain(header_parameters))
    }
}

impl Endpoint<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<Endpoint<Reference>, UnknownReference> {
        let Self {
            path,
            endpoint_documentation,
            operation_documentation,
            verb,
            operation_id,
            request_body,
            response,
            query_parameters,
            path_parameters,
            headers,
        } = self;

        let headers = headers
            .into_iter()
            .map(|(name, param)| Ok((name, param.resolve_refs(&resolver)?)))
            .collect::<Result<_, _>>()?;
        let path_parameters = path_parameters.as_ref().map(&resolver).transpose()?;
        let query_parameters = query_parameters.as_ref().map(&resolver).transpose()?;
        let request_body = request_body.as_ref().map(&resolver).transpose()?;
        let response = resolver(&response)?;

        Ok(Endpoint {
            path,
            endpoint_documentation,
            operation_documentation,
            verb,
            operation_id,
            request_body,
            response,
            headers,
            path_parameters,
            query_parameters,
        })
    }
}

impl Endpoint {
    /// Generate an item definition for this item.
    ///
    /// The name resolver should be able to efficiently extract item names from references.
    pub fn emit<'a>(
        &self,
        api_model: &ApiModel,
        name_resolver: impl Fn(Reference) -> Result<&'a str, UnknownReference>,
    ) -> Result<TokenStream, UnknownReference> {
        let docs = self.doc_string();
        let docs = quote!(#[doc = #docs]);
        // todo: proper suffix
        let function_name = make_ident(&self.function_name(None));

        let parameters = self
            .function_parameters(api_model)?
            .map(|(name, ref_, required)| {
                let param_name = make_ident(&name);
                let mut type_name = api_model.definition(ref_, &name_resolver)?;
                if !required {
                    type_name = quote!(Option< #type_name >);
                }
                Ok(quote!(#param_name: #type_name))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let request_body = self
            .request_body
            .map(|ref_| {
                let type_name = api_model.definition(ref_, &name_resolver)?;

                Ok(quote!(request_body: #type_name))
            })
            .transpose()?;

        let response_body = api_model.definition(self.response, name_resolver)?;

        Ok(quote! {
            #docs
            async fn #function_name (
                &self,
                #(
                    #parameters,
                )*
                #request_body
            ) -> #response_body;
        })
    }
}

fn make_operation_spec_name(
    operation_id: Option<&str>,
    operation_type: &str,
    verb: Verb,
    path: &str,
) -> String {
    match operation_id {
        Some(operation_id) => format!("{}{operation_type}", AsUpperCamelCase(operation_id)),
        None => format!(
            "{}{}{operation_type}",
            AsUpperCamelCase(verb.to_string()),
            AsUpperCamelCase(path)
        ),
    }
}

/// Compute an appropriate name for this endpoint, but do not apply casing rules to it
fn compute_uncased_item_name(
    operation_id: Option<&str>,
    verb: Verb,
    path: &str,
    suffix: Option<&str>,
) -> String {
    operation_id.map(ToOwned::to_owned).unwrap_or_else(|| {
        let mut name = format!("{verb} {path}");
        if let Some(suffix) = suffix {
            name.push(' ');
            name.push_str(suffix);
        }
        name
    })
}

/// Make a parameter object
fn make_param_object(
    model: &mut ApiModel<Ref>,
    param_type: &str,
    uncased_item_name: &str,
    params: Vec<Parameter<Ref>>,
) -> Option<Ref> {
    (!params.is_empty()).then(|| {
        let members = params
            .into_iter()
            .map(|param| {
                let Parameter {
                    spec_name,
                    required,
                    item_ref,
                    ..
                } = param;
                let mut member = ObjectMember::new(item_ref);
                member.inline_option = !required;
                (spec_name, member)
            })
            .collect();

        let value = Object {
            members,
            is_generated_body_and_headers: false,
        }
        .into();

        let mut rust_name = format!(
            "{}{}Parameters",
            uncased_item_name.to_upper_camel_case(),
            param_type.to_upper_camel_case()
        );
        model.deconflict_ident(&mut rust_name);

        let item = Item {
            docs: Some(format!(
                "Combination item for {param_type} parameters of `{uncased_item_name}`"
            )),
            rust_name,
            value,
            ..Default::default()
        };

        model
            .add_item(item, None)
            .expect("`add_item` cannot fail with `None` reference_name")
    })
}

/// Iterate over the OpenApi specification, constructing endpoints anad inserting each into the model.
pub(crate) fn insert_endpoints(spec: &OpenAPI, model: &mut ApiModel<Ref>) -> Result<(), Error> {
    for (path, path_item) in path_items(spec) {
        for (verb, operation) in path_item.iter() {
            let verb: Verb = verb.parse().map_err(|err| Error::UnknownVerb {
                verb: verb.to_string(),
                err,
            })?;
            let endpoint_documentation = path_item
                .description
                .as_deref()
                .or(path_item.summary.as_deref())
                .map(ToOwned::to_owned);
            let operation_documentation = operation
                .description
                .as_deref()
                .or(operation.summary.as_deref())
                .map(ToOwned::to_owned);

            // we need to ensure that when at least one response variant has the same response code and more than one
            // content type, the generated function includes the "Accept" header. This may or may not appear in the
            // spec, so we ensure it appears first in the parameter list. After that point, if it does appear in the user spec,
            // we can trust `IndexMap`'s update implementation to ensure it remains first in the emitted list.
            let accept_header = response::has_responses_distinguished_only_by_content_type(
                spec, operation,
            )
            .then(|| {
                let format = openapiv3::ParameterSchemaOrContent::Schema(
                    openapiv3::ReferenceOr::Item(openapiv3::Schema {
                        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(
                            openapiv3::StringType {
                                format: openapiv3::VariantOrUnknownOrEmpty::Unknown(
                                    "accept-header".into(),
                                ),
                                ..Default::default()
                            },
                        )),
                        schema_data: Default::default(),
                    }),
                );

                let parameter = openapiv3::Parameter::Header {
                    parameter_data: openapiv3::ParameterData {
                        name: "accept".into(),
                        description: Some("The content type expected by the client".into()),
                        required: false,
                        format,
                        deprecated: Default::default(),
                        example: Default::default(),
                        examples: Default::default(),
                        explode: Default::default(),
                        extensions: Default::default(),
                    },
                    style: Default::default(),
                };

                openapiv3::ReferenceOr::Item(parameter)
            });
            let accept_header = accept_header.iter();

            // first real params are the path item parameters
            let path_item_params = path_item.parameters.iter();

            // update with the operation parameters
            let operation_params = operation.parameters.iter();

            let mut path_parameters = Vec::new();
            let mut query_parameters = Vec::new();
            let mut headers = IndexMap::new();

            // `IndexMap::from_iter` uses the same logic as its `extend`,
            // which lets subsequent items override earlier items.
            for maybe_param in accept_header
                .chain(path_item_params)
                .chain(operation_params)
                .map(|param_ref| convert_param_ref(spec, model, param_ref))
            {
                let param = maybe_param?;
                match param.location {
                    ParameterLocation::Path => path_parameters.push(param),
                    ParameterLocation::Query => query_parameters.push(param),
                    ParameterLocation::Header => {
                        headers.insert(param.rust_name.clone(), param);
                    }
                    ParameterLocation::Cookie => return Err(Error::CookesAreNotSupported),
                }
            }

            let uncased_item_name =
                compute_uncased_item_name(operation.operation_id.as_deref(), verb, path, None);

            let path_parameters =
                make_param_object(model, "path", &uncased_item_name, path_parameters);

            let query_parameters =
                make_param_object(model, "query", &uncased_item_name, query_parameters);

            let operation_id = operation.operation_id.clone();

            let request_body = {
                let spec_name =
                    make_operation_spec_name(operation_id.as_deref(), "Request", verb, path);
                operation
                    .request_body
                    .as_ref()
                    .filter(|_request_body| verb.request_body_is_legal())
                    .map(|body_ref| {
                        request_body::create_request_body_from_ref(
                            spec, model, &spec_name, body_ref,
                        )
                    })
                    .transpose()?
            };

            let response = {
                let spec_name =
                    make_operation_spec_name(operation_id.as_deref(), "Response", verb, path);
                response::create_responses(spec, model, &spec_name, &operation.responses)?
            };

            let endpoint = Endpoint::<Ref> {
                path: path.to_string(),
                endpoint_documentation,
                operation_documentation,
                verb,
                headers,
                path_parameters,
                query_parameters,
                operation_id,
                request_body,
                response,
            };

            model.endpoints.push(endpoint);
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown http verb")]
    UnknownVerb {
        verb: String,
        #[source]
        err: strum::ParseError,
    },
    #[error("could not create reference from supplied parameter ref")]
    ConvertParamRef(#[source] anyhow::Error),
    #[error("could not create from supplied request body")]
    CreateRequestBody(#[source] anyhow::Error),
    #[error("could not create from supplied response")]
    CreateResponse(#[source] anyhow::Error),
    #[error("cookies are not supported")]
    CookesAreNotSupported,
}
