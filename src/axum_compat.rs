use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::AsStatusCode;

/// Construct a [`Response`][axum::response::Response] from the the default response type for an endpoint.
#[inline]
pub fn default_response<D>(default: D) -> Response
where
    D: AsStatusCode + Serialize,
{
    let status = default.as_status_code();
    (status, Json(default)).into_response()
}
