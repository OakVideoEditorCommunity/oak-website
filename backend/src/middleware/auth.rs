use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use crate::state::AppState;

/// Validates that the request carries the expected Bearer token.
fn bearer_matches(header: Option<&str>, expected: &str) -> bool {
    header
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|token| token == expected)
        .unwrap_or(false)
}

/// Axum middleware that protects admin routes with a Bearer token.
pub async fn admin_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let expected = &state.config.admin.token;
    if expected.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    if bearer_matches(auth_header, expected) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_bearer_token_matches() {
        assert!(bearer_matches(Some("Bearer secret-token"), "secret-token"));
    }

    #[test]
    fn missing_header_fails() {
        assert!(!bearer_matches(None, "token"));
    }

    #[test]
    fn wrong_scheme_fails() {
        assert!(!bearer_matches(Some("Basic dXNlcjpwYXNz"), "token"));
    }

    #[test]
    fn wrong_token_fails() {
        assert!(!bearer_matches(Some("Bearer wrong"), "token"));
    }

    #[test]
    fn empty_token_after_bearer_fails() {
        assert!(!bearer_matches(Some("Bearer "), "token"));
    }
}
