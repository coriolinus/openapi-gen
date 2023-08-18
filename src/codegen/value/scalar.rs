use proc_macro2::TokenStream;
use quote::quote;

/// Scalar types known to this type generator. These types are effectively primitives.
///
/// Because this varies with the set of features enabled, it can't be treated as exhaustive.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum Scalar {
    Unit,
    F64,
    F32,
    I64,
    I32,
    U64,
    U32,
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
    #[cfg(feature = "integer-restrictions")]
    BoundedU32(u32, u32),
    #[cfg(feature = "integer-restrictions")]
    BoundedU64(u64, u64),
    #[cfg(feature = "api-problem")]
    ApiProblem,
    Mime,
    AcceptHeader,
}

impl Scalar {
    /// Should a newtype derived from this scalar impl `Eq`?
    pub fn impls_eq(self) -> bool {
        match self {
            Scalar::F64 | Scalar::F32 | Scalar::Any => false,
            Scalar::Unit
            | Scalar::I64
            | Scalar::I32
            | Scalar::U32
            | Scalar::U64
            | Scalar::String
            | Scalar::Binary
            | Scalar::Date
            | Scalar::DateTime
            | Scalar::IpAddr
            | Scalar::Ipv4Addr
            | Scalar::Ipv6Addr
            | Scalar::Mime
            | Scalar::AcceptHeader
            | Scalar::Bool => true,
            #[cfg(feature = "bytes")]
            Scalar::Bytes => true,
            #[cfg(feature = "uuid")]
            Scalar::Uuid => true,
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI32(_, _)
            | Scalar::BoundedI64(_, _)
            | Scalar::BoundedU32(_, _)
            | Scalar::BoundedU64(_, _) => true,
            #[cfg(feature = "api-problem")]
            Scalar::ApiProblem => true,
        }
    }

    /// Should a newtype derived from this scalar impl `Copy`?
    pub fn impls_copy(self) -> bool {
        match self {
            Scalar::Unit
            | Scalar::F64
            | Scalar::F32
            | Scalar::I64
            | Scalar::I32
            | Scalar::U32
            | Scalar::U64
            | Scalar::Date
            | Scalar::DateTime
            | Scalar::IpAddr
            | Scalar::Ipv4Addr
            | Scalar::Ipv6Addr
            | Scalar::Bool => true,
            Scalar::String | Scalar::Binary | Scalar::Any | Scalar::Mime | Scalar::AcceptHeader => {
                false
            }
            #[cfg(feature = "bytes")]
            Scalar::Bytes => false,
            #[cfg(feature = "uuid")]
            Scalar::Uuid => true,
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI32(_, _)
            | Scalar::BoundedI64(_, _)
            | Scalar::BoundedU32(_, _)
            | Scalar::BoundedU64(_, _) => true,
            #[cfg(feature = "api-problem")]
            Scalar::ApiProblem => false,
        }
    }

    pub fn impls_hash(self) -> bool {
        match self {
            Scalar::F64 | Scalar::F32 | Scalar::Any => false,
            Scalar::Unit
            | Scalar::I64
            | Scalar::I32
            | Scalar::U32
            | Scalar::U64
            | Scalar::String
            | Scalar::Binary
            | Scalar::Date
            | Scalar::DateTime
            | Scalar::IpAddr
            | Scalar::Ipv4Addr
            | Scalar::Ipv6Addr
            | Scalar::Mime
            | Scalar::AcceptHeader
            | Scalar::Bool => true,
            #[cfg(feature = "bytes")]
            Scalar::Bytes => true,
            #[cfg(feature = "uuid")]
            Scalar::Uuid => true,
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedI32(_, _)
            | Scalar::BoundedI64(_, _)
            | Scalar::BoundedU32(_, _)
            | Scalar::BoundedU64(_, _) => true,
            #[cfg(feature = "api-problem")]
            Scalar::ApiProblem => false,
        }
    }

    pub fn emit_type(self) -> TokenStream {
        match self {
            Scalar::Unit => quote!(()),
            Scalar::Bool => quote!(bool),
            Scalar::F64 => quote!(f64),
            Scalar::F32 => quote!(f32),
            Scalar::I64 => quote!(i64),
            Scalar::I32 => quote!(i32),
            Scalar::U32 => quote!(u32),
            Scalar::U64 => quote!(u64),
            Scalar::String => quote!(String),
            Scalar::Binary => quote!(Vec<u8>),
            Scalar::Date => quote!(openapi_gen::reexport::time::Date),
            Scalar::DateTime => quote!(openapi_gen::reexport::time::OffsetDateTime),
            Scalar::IpAddr => quote!(std::net::IpAddr),
            Scalar::Ipv4Addr => quote!(std::net::Ipv4Addr),
            Scalar::Ipv6Addr => quote!(std::net::Ipv6Addr),
            Scalar::Any => quote!(openapi_gen::reexport::serde_json::Value),
            Scalar::Mime => quote!(openapi_gen::reexport::mime::Mime),
            Scalar::AcceptHeader => quote!(openapi_gen::reexport::accept_header::Accept),
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
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedU32(min, max) => {
                quote!(openapi_gen::reexport::bounded_integer::BoundedU32<#min, #max>)
            }
            #[cfg(feature = "integer-restrictions")]
            Scalar::BoundedU64(min, max) => {
                quote!(openapi_gen::reexport::bounded_integer::BoundedU64<#min, #max>)
            }
            #[cfg(feature = "api-problem")]
            Scalar::ApiProblem => quote!(openapi_gen::reexport::http_api_problem::HttpApiProblem),
        }
    }

    pub fn integer_32_from(integer_type: &openapiv3::IntegerType) -> Self {
        if integer_type.minimum == Some(0) && integer_type.maximum.is_none() {
            return Self::U32;
        }

        #[cfg(feature = "integer-restrictions")]
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

            return Self::BoundedI32(min, max);
        }

        Self::I32
    }

    pub fn integer_64_from(integer_type: &openapiv3::IntegerType) -> Self {
        if integer_type.minimum == Some(0) && integer_type.maximum.is_none() {
            return Self::U64;
        }

        #[cfg(feature = "integer-restrictions")]
        if integer_type.minimum.is_some() || integer_type.maximum.is_some() {
            let mut min = integer_type.minimum.unwrap_or(i64::MIN);
            if integer_type.exclusive_minimum {
                min += 1;
            }

            let mut max = integer_type.maximum.unwrap_or(i64::MAX);
            if integer_type.exclusive_maximum {
                max -= 1;
            }

            return Self::BoundedI64(min, max);
        }

        Self::I64
    }
}
