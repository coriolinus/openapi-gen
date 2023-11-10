#![allow(non_camel_case_types)]
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    openapi_gen::reexport::derive_more::Constructor
)]
#[serde(crate = "openapi_gen::reexport::serde")]
pub struct Container {
    pub number: f64,
    pub number_float: f32,
    pub number_double: f64,
    pub integer: i64,
    pub integer_i32: i32,
    pub integer_i64: i64,
    pub unsigned_integer: u64,
    pub unsigned_integer_i32: u32,
    pub unsigned_integer_i64: u64,
    pub bounded_integer_low: openapi_gen::reexport::bounded_integer::BoundedI64<
        1i64,
        9223372036854775807i64,
    >,
    pub bounded_integer_high: openapi_gen::reexport::bounded_integer::BoundedI64<
        -9223372036854775808i64,
        5i64,
    >,
    pub bounded_integer_both: openapi_gen::reexport::bounded_integer::BoundedI64<
        0i64,
        5i64,
    >,
    pub exclusive_bounded_integer_low: openapi_gen::reexport::bounded_integer::BoundedI64<
        2i64,
        9223372036854775807i64,
    >,
    pub exclusive_bounded_integer_high: openapi_gen::reexport::bounded_integer::BoundedI64<
        -9223372036854775808i64,
        4i64,
    >,
    pub exclusive_bounded_integer_both: openapi_gen::reexport::bounded_integer::BoundedI64<
        1i64,
        4i64,
    >,
    pub bounded_integer_low_i32: openapi_gen::reexport::bounded_integer::BoundedI32<
        1i32,
        2147483647i32,
    >,
    pub bounded_integer_high_i32: openapi_gen::reexport::bounded_integer::BoundedI32<
        -2147483648i32,
        5i32,
    >,
    pub bounded_integer_both_i32: openapi_gen::reexport::bounded_integer::BoundedI32<
        0i32,
        5i32,
    >,
    pub exclusive_bounded_integer_low_i32: openapi_gen::reexport::bounded_integer::BoundedI32<
        2i32,
        2147483647i32,
    >,
    pub exclusive_bounded_integer_high_i32: openapi_gen::reexport::bounded_integer::BoundedI32<
        -2147483648i32,
        4i32,
    >,
    pub exclusive_bounded_integer_both_i32: openapi_gen::reexport::bounded_integer::BoundedI32<
        1i32,
        4i32,
    >,
    pub string: String,
    pub string_binary: Vec<u8>,
    pub string_byte: openapi_gen::Bytes,
    pub string_base64: openapi_gen::Bytes,
    pub string_date: openapi_gen::reexport::time::Date,
    pub string_datetime: openapi_gen::reexport::time::OffsetDateTime,
    pub string_ip: std::net::IpAddr,
    pub string_ipv4: std::net::Ipv4Addr,
    pub string_ipv6: std::net::Ipv6Addr,
    pub string_uuid: openapi_gen::reexport::uuid::Uuid,
    pub string_unrecognized: String,
}
#[openapi_gen::reexport::async_trait::async_trait]
pub trait Api {}
/// Transform an instance of [`trait Api`][Api] into a [`Router`][axum::Router].
pub fn build_router<Instance>(instance: Instance) -> openapi_gen::reexport::axum::Router
where
    Instance: 'static + Api + Send + Sync,
{
    #[allow(unused_variables)]
    let instance = ::std::sync::Arc::new(instance);
    openapi_gen::reexport::axum::Router::new()
}

