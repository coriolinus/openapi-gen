use proc_macro2::TokenStream;
use quote::quote;

/// Scalar types known to this type generator. These types are effectively primitives.
///
/// Because this varies with the set of features enabled, it can't be treated as exhaustive.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Scalar {
    F64,
    F32,
    I64,
    I32,
    String,
    Binary,
    #[cfg(feature = "bytes")]
    Bytes,
    Date,
    DateTime,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    #[cfg(feature = "uuid")]
    Uuid,
    Bool,
    Any,
    #[cfg(feature = "integer-restrictions")]
    BoundedI32(i32, i32),
    #[cfg(feature = "integer-restrictions")]
    BoundedI64(i64, i64),
}

impl Scalar {
    /// Should a newtype derived from this scalar impl `Eq`?
    pub fn impls_eq(self) -> bool {
        match self {
            Scalar::F64 | Scalar::F32 | Scalar::Any => false,
            Scalar::I64
            | Scalar::I32
            | Scalar::String
            | Scalar::Binary
            | Scalar::Date
            | Scalar::DateTime
            | Scalar::IpAddr
            | Scalar::Ipv4Addr
            | Scalar::Ipv6Addr
            | Scalar::Bool => true,
            #[cfg(feature = "bytes")]
            Scalar::Bytes => true,
            #[cfg(feature = "uuid")]
            Scalar::Uuid => true,
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI32(_, _) | Scalar::BoundedI64(_, _) => true,
        }
    }

    /// Should a newtype derived from this scalar impl `Copy`?
    pub fn impls_copy(self) -> bool {
        match self {
            Scalar::F64
            | Scalar::F32
            | Scalar::I64
            | Scalar::I32
            | Scalar::Date
            | Scalar::DateTime
            | Scalar::IpAddr
            | Scalar::Ipv4Addr
            | Scalar::Ipv6Addr
            | Scalar::Bool => true,
            Scalar::String | Scalar::Binary | Scalar::Any => false,
            #[cfg(feature = "bytes")]
            Scalar::Bytes => false,
            #[cfg(feature = "uuid")]
            Scalar::Uuid => true,
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI32(_, _) | Scalar::BoundedI64(_, _) => true,
        }
    }

    pub fn impls_hash(self) -> bool {
        match self {
            Scalar::F64 | Scalar::F32 | Scalar::Any => false,
            Scalar::I64
            | Scalar::I32
            | Scalar::String
            | Scalar::Binary
            | Scalar::Date
            | Scalar::DateTime
            | Scalar::IpAddr
            | Scalar::Ipv4Addr
            | Scalar::Ipv6Addr
            | Scalar::Bool => true,
            #[cfg(feature = "bytes")]
            Scalar::Bytes => true,
            #[cfg(feature = "uuid")]
            Scalar::Uuid => true,
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI32(_, _) | Scalar::BoundedI64(_, _) => true,
        }
    }

    pub fn emit_type(self) -> TokenStream {
        match self {
            Scalar::Bool => quote!(bool),
            Scalar::F64 => quote!(f64),
            Scalar::F32 => quote!(f32),
            Scalar::I64 => quote!(i64),
            Scalar::I32 => quote!(i32),
            Scalar::String => quote!(String),
            Scalar::Binary => quote!(Vec<u8>),
            Scalar::Date => quote!(openapi_gen::reexport::time::Date),
            Scalar::DateTime => quote!(openapi_gen::reexport::time::OffsetDateTime),
            Scalar::IpAddr => quote!(std::net::IpAddr),
            Scalar::Ipv4Addr => quote!(std::net::Ipv4Addr),
            Scalar::Ipv6Addr => quote!(std::net::Ipv6Addr),
            Scalar::Any => quote!(openapi_gen::reexport::serde_json::Value),
            #[cfg(feature = "bytes")]
            Scalar::Bytes => quote!(openapi_gen::Bytes),
            #[cfg(feature = "uuid")]
            Scalar::Uuid => quote!(openapi_gen::reexport::uuid::Uuid),
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI32(min, max) => {
                quote!(openapi_gen::reexport::bounded_integer::BoundedI32<#min, #max>)
            }
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI64(min, max) => {
                quote!(openapi_gen::reexport::bounded_integer::BoundedI64<#min, #max>)
            }
        }
    }

    #[cfg(feature = "integer-restrictions")]
    pub fn i32_from(integer_type: &openapiv3::IntegerType) -> Self {
        if integer_type.minimum.is_some() || integer_type.maximum.is_some() {
            let mut min = integer_type
                .minimum
                .and_then(|min| min.try_into().ok())
                .unwrap_or(i32::MIN);
            if integer_type.exclusive_minimum {
                min += 1;
            }

            let mut max = integer_type
                .maximum
                .and_then(|max| max.try_into().ok())
                .unwrap_or(i32::MAX);
            if integer_type.exclusive_maximum {
                max -= 1;
            }

            Self::BoundedI32(min, max)
        } else {
            Self::I32
        }
    }

    #[cfg(feature = "integer-restrictions")]
    pub fn i64_from(integer_type: &openapiv3::IntegerType) -> Self {
        if integer_type.minimum.is_some() || integer_type.maximum.is_some() {
            let mut min = integer_type.minimum.unwrap_or(i64::MIN);
            if integer_type.exclusive_minimum {
                min += 1;
            }

            let mut max = integer_type.maximum.unwrap_or(i64::MAX);
            if integer_type.exclusive_maximum {
                max -= 1;
            }

            Self::BoundedI64(min, max)
        } else {
            Self::I64
        }
    }
}
