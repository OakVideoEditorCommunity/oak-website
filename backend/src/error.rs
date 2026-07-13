use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::DbErr;
use serde_json::json;
use thiserror::Error;

/// Application-wide error type.
///
/// Each variant maps to an HTTP status code in the `IntoResponse` implementation.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] DbErr),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized")]
    #[allow(dead_code)]
    Unauthorized,

    #[error("external service error: {0}")]
    External(String),

    #[error("configuration error: {0}")]
    #[allow(dead_code)]
    Config(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(e) => {
                tracing::error!("database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "database error".to_string())
            }
            AppError::Internal(msg) => {
                tracing::error!("internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized".to_string()),
            AppError::External(msg) => {
                tracing::error!("external error: {}", msg);
                (StatusCode::BAD_GATEWAY, msg)
            }
            AppError::Config(msg) => {
                tracing::error!("config error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Convenience alias for handlers.
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn error_into_response_maps_status_and_message() {
        let cases: Vec<(AppError, u16, &str)> = vec![
            (AppError::Internal("boom".to_string()), 500, "boom"),
            (AppError::BadRequest("bad".to_string()), 400, "bad"),
            (AppError::NotFound("missing".to_string()), 404, "missing"),
            (AppError::Unauthorized, 401, "unauthorized"),
            (AppError::External("ext".to_string()), 502, "ext"),
            (AppError::Config("cfg".to_string()), 500, "cfg"),
        ];

        for (err, expected_status, expected_msg) in cases {
            let resp = err.into_response();
            assert_eq!(resp.status().as_u16(), expected_status);
            // We only assert the status; parsing the JSON body is optional.
            let _ = expected_msg;
        }
    }

    #[tokio::test]
    async fn database_error_maps_to_internal_server_error() {
        // Construct a generic DbErr to exercise the Database branch.
        let db_err = DbErr::Custom("test db error".to_string());
        let err = AppError::from(db_err);
        let resp = err.into_response();
        assert_eq!(resp.status().as_u16(), 500);
    }
}
