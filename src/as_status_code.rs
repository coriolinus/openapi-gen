use http::StatusCode;

/// This trait should be implemented for types which carry their status code internally.
///
/// This permits them to be used in a context where the appropriate context is not obvious,
/// such as the `default` return value for an endpoint.
pub trait AsStatusCode {
    fn as_status_code(&self) -> StatusCode;
}

#[cfg(feature = "http-api-problem")]
impl AsStatusCode for http_api_problem::HttpApiProblem {
    fn as_status_code(&self) -> StatusCode {
        self.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
