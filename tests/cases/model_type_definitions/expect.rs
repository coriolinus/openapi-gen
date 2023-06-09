///should be `pub type Count = u64`
pub type Count = u64;
///should be `pub struct FirstBar(pub String);`
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    Eq,
    Hash,
    openapi_gen::reexport::derive_more::From,
    openapi_gen::reexport::derive_more::Into,
    openapi_gen::reexport::derive_more::Deref,
    openapi_gen::reexport::derive_more::DerefMut,
    openapi_gen::reexport::derive_more::Constructor
)]
pub struct FirstBar(pub String);
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
pub struct Foo {
    ///should be `pub type Count = u64`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty_of_bar: Option<Count>,
    ///should be `pub struct FirstBar(pub String);`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_bar: Option<FirstBar>,
}

