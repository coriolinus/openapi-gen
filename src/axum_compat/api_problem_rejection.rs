use axum::{extract::rejection::JsonRejection, response::IntoResponse};
use http_api_problem::{HttpApiProblem, StatusCode};

#[derive(Debug)]
pub struct ApiProblemRejection(pub HttpApiProblem);

impl From<JsonRejection> for ApiProblemRejection {
    fn from(value: JsonRejection) -> Self {
        let status = match value {
            JsonRejection::JsonDataError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            JsonRejection::JsonSyntaxError(_) => StatusCode::BAD_REQUEST,
            JsonRejection::MissingJsonContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            _ => StatusCode::BAD_REQUEST,
        };
        let title = value.to_string();

        let mut err = &value as &dyn std::error::Error;
        let mut detail = String::new();
        while let Some(predecessor) = err.source() {
            detail.extend(format!("{predecessor}; ").chars());
            err = predecessor;
        }

        Self(HttpApiProblem::new(status).title(title).detail(detail))
    }
}

impl IntoResponse for ApiProblemRejection {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}
