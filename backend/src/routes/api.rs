use axum::{routing::get, Router};

use crate::{
    handlers::{docs, health, releases},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health))
        .route("/releases", get(releases::list_releases))
        .route("/releases/latest", get(releases::latest_release))
        .route("/releases/:id", get(releases::get_release))
        .route("/releases/:id/download", get(releases::download_release))
        .route("/docs", get(docs::list_docs))
        .route("/docs/versions", get(docs::list_versions))
        .route("/docs/:lang/:slug", get(docs::get_doc))
        .route("/docs/:version/:lang/:slug", get(docs::get_versioned_doc))
}
