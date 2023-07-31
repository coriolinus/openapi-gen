type NotNullableAndRequired = i64;
type NotNullableAndNotRequired = i64;
type MaybeNullableAndRequired = Option<NullableAndRequired>;
type NullableAndRequired = i64;
type MaybeNullableAndNotRequired = Option<NullableAndNotRequired>;
///note that this produces an `Option<Option<_>>`
type NullableAndNotRequired = i64;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Copy,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct Foo {
    pub not_nullable_and_required: NotNullableAndRequired,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_nullable_and_not_required: Option<NotNullableAndNotRequired>,
    pub nullable_and_required: MaybeNullableAndRequired,
    ///note that this produces an `Option<Option<_>>`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nullable_and_not_required: Option<MaybeNullableAndNotRequired>,
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {}

