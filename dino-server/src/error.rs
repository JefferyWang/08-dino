use axum::response::Response;
use axum::{http::Method, response::IntoResponse};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Host not found: {0}")]
    HostNotFound(String),

    #[error("Path not found: {0}")]
    RoutePathNotFound(String),

    #[error("Method not found: {0}")]
    RouteMethodNotAllowed(Method),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Serde json error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = match self {
            AppError::HostNotFound(_) => axum::http::StatusCode::NOT_FOUND,
            AppError::RoutePathNotFound(_) => axum::http::StatusCode::NOT_FOUND,
            AppError::RouteMethodNotAllowed(_) => axum::http::StatusCode::METHOD_NOT_ALLOWED,
            AppError::Anyhow(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Serde(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, self.to_string()).into_response()
    }
}
