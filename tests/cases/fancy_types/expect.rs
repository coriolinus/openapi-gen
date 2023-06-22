type Number = f64;
type NumberFloat = f32;
type NumberDouble = f64;
type Integer = i64;
type IntegerI32 = i32;
type IntegerI64 = i64;
type UnsignedInteger = u64;
type UnsignedIntegerI32 = u32;
type UnsignedIntegerI64 = u64;
type BoundedIntegerLow = openapi_gen::reexport::bounded_integer::BoundedI64<
    1i64,
    9223372036854775807i64,
>;
type BoundedIntegerHigh = openapi_gen::reexport::bounded_integer::BoundedI64<
    -9223372036854775808i64,
    5i64,
>;
type BoundedIntegerBoth = openapi_gen::reexport::bounded_integer::BoundedI64<0i64, 5i64>;
type ExclusiveBoundedIntegerLow = openapi_gen::reexport::bounded_integer::BoundedI64<
    2i64,
    9223372036854775807i64,
>;
type ExclusiveBoundedIntegerHigh = openapi_gen::reexport::bounded_integer::BoundedI64<
    -9223372036854775808i64,
    4i64,
>;
type ExclusiveBoundedIntegerBoth = openapi_gen::reexport::bounded_integer::BoundedI64<
    1i64,
    4i64,
>;
type BoundedIntegerLowI32 = openapi_gen::reexport::bounded_integer::BoundedI32<
    1i32,
    2147483647i32,
>;
type BoundedIntegerHighI32 = openapi_gen::reexport::bounded_integer::BoundedI32<
    -2147483648i32,
    5i32,
>;
type BoundedIntegerBothI32 = openapi_gen::reexport::bounded_integer::BoundedI32<
    0i32,
    5i32,
>;
type ExclusiveBoundedIntegerLowI32 = openapi_gen::reexport::bounded_integer::BoundedI32<
    2i32,
    2147483647i32,
>;
type ExclusiveBoundedIntegerHighI32 = openapi_gen::reexport::bounded_integer::BoundedI32<
    -2147483648i32,
    4i32,
>;
type ExclusiveBoundedIntegerBothI32 = openapi_gen::reexport::bounded_integer::BoundedI32<
    1i32,
    4i32,
>;
type String_ = String;
type StringBinary = Vec<u8>;
type StringByte = openapi_gen::Bytes;
type StringBase64 = openapi_gen::Bytes;
type StringDate = openapi_gen::reexport::time::Date;
type StringDatetime = openapi_gen::reexport::time::OffsetDateTime;
type StringIp = std::net::IpAddr;
type StringIpv4 = std::net::Ipv4Addr;
type StringIpv6 = std::net::Ipv6Addr;
type StringUuid = openapi_gen::reexport::uuid::Uuid;
type StringUnrecognized = String;
#[derive(
    Debug,
    Clone,
    PartialEq,
    openapi_gen::reexport::serde::Serialize,
    openapi_gen::reexport::serde::Deserialize,
    openapi_gen::reexport::derive_more::Constructor
)]
pub struct Container {
    number: Number,
    number_float: NumberFloat,
    number_double: NumberDouble,
    integer: Integer,
    integer_i32: IntegerI32,
    integer_i64: IntegerI64,
    unsigned_integer: UnsignedInteger,
    unsigned_integer_i32: UnsignedIntegerI32,
    unsigned_integer_i64: UnsignedIntegerI64,
    bounded_integer_low: BoundedIntegerLow,
    bounded_integer_high: BoundedIntegerHigh,
    bounded_integer_both: BoundedIntegerBoth,
    exclusive_bounded_integer_low: ExclusiveBoundedIntegerLow,
    exclusive_bounded_integer_high: ExclusiveBoundedIntegerHigh,
    exclusive_bounded_integer_both: ExclusiveBoundedIntegerBoth,
    bounded_integer_low_i32: BoundedIntegerLowI32,
    bounded_integer_high_i32: BoundedIntegerHighI32,
    bounded_integer_both_i32: BoundedIntegerBothI32,
    exclusive_bounded_integer_low_i32: ExclusiveBoundedIntegerLowI32,
    exclusive_bounded_integer_high_i32: ExclusiveBoundedIntegerHighI32,
    exclusive_bounded_integer_both_i32: ExclusiveBoundedIntegerBothI32,
    string: String_,
    string_binary: StringBinary,
    string_byte: StringByte,
    string_base64: StringBase64,
    string_date: StringDate,
    string_datetime: StringDatetime,
    string_ip: StringIp,
    string_ipv4: StringIpv4,
    string_ipv6: StringIpv6,
    string_uuid: StringUuid,
    string_unrecognized: StringUnrecognized,
}
