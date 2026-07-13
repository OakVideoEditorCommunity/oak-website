use axum::{
    middleware,
    routing::{post},
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
        .route_layer(middleware::from_fn_with_state(state, admin_auth))
}
