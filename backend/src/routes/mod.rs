use axum::Router;

use crate::state::AppState;

pub mod admin;
pub mod api;

pub fn build(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/api/v1", api::router())
        .nest("/api/admin", admin::router(state))
}
