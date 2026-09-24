use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    handlers::admin,
    middleware::auth::admin_auth,
    state::AppState,
};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/releases/sync", post(admin::sync_releases))
        .route("/bug-reports", get(admin::list_bug_reports))
        .route_layer(middleware::from_fn_with_state(state, admin_auth))
}
