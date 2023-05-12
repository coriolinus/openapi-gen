mod binary;
pub use binary::Binary;

#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "bytes")]
pub use bytes::Bytes;
