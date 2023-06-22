///who this gift is for
type For = String;
/**who this gift is from.

May be omitted for anonymous gifting.
*/
type From_ = String;
/**a teaser message to excite the imagination before opening the gift.

The point is to see if the rename attribute is emitted appropriately if the
default casing is unexpected.
*/
type Message = String;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
pub struct GiftTag {
    ///who this gift is for
    #[serde(rename = "for")]
    pub for_: For,
    /**who this gift is from.

May be omitted for anonymous gifting.
*/
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<From_>,
    /**a teaser message to excite the imagination before opening the gift.

The point is to see if the rename attribute is emitted appropriately if the
default casing is unexpected.
*/
    #[serde(rename = "Message", skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,
}

