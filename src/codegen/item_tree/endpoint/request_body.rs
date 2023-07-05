use indexmap::IndexMap;
use openapiv3::ReferenceOr;

use crate::ApiModel;

use super::super::api_model::{Ref, Reference, UnknownReference};

#[derive(Debug, Clone)]
pub struct RequestBody<Ref = Reference> {
    pub description: Option<String>,
    /// Request body data by content type.
    pub content: IndexMap<String, Ref>,
    pub required: bool,
}

impl RequestBody<Ref> {
    pub(crate) fn resolve_refs(
        self,
        resolver: impl Fn(&Ref) -> Result<Reference, UnknownReference>,
    ) -> Result<RequestBody<Reference>, UnknownReference> {
        let Self {
            description,
            content,
            required,
        } = self;

        let content = content
            .into_iter()
            .map(|(content_type, ref_)| Ok((content_type, resolver(&ref_)?)))
            .collect::<Result<_, _>>()?;

        Ok(RequestBody {
            description,
            content,
            required,
        })
    }

    pub(crate) fn try_new(
        body_ref: &ReferenceOr<openapiv3::RequestBody>,
        model: &mut ApiModel<Ref>,
    ) -> Result<Self, super::Error> {
        todo!()
    }
}
