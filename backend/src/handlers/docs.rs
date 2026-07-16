use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    error::{AppError, AppResult},
    models::{DocPageResponse, DocsIndexQuery, DocsIndexResponse, DocsVersionsResponse},
    state::AppState,
};

pub async fn list_docs(
    State(state): State<AppState>,
    Query(query): Query<DocsIndexQuery>,
) -> AppResult<Json<DocsIndexResponse>> {
    let docs = state
        .docs
        .read()
        .map_err(|e| AppError::Internal(format!("docs index lock poisoned: {}", e)))?;
    let version = match query.version {
        Some(version) => {
            if !docs.has_version(&version) {
                return Err(AppError::NotFound(format!(
                    "unknown docs version '{}'",
                    version
                )));
            }
            version
        }
        None => docs.default_version().unwrap_or_default(),
    };

    Ok(Json(DocsIndexResponse {
        zh: docs.list(&version, "zh"),
        en: docs.list(&version, "en"),
        version,
    }))
}

pub async fn get_doc(
    State(state): State<AppState>,
    Path((lang, slug)): Path<(String, String)>,
) -> AppResult<Json<DocPageResponse>> {
    let docs = state
        .docs
        .read()
        .map_err(|e| AppError::Internal(format!("docs index lock poisoned: {}", e)))?;
    let version = docs.default_version().unwrap_or_default();
    let page = docs.get(&version, &lang, &slug).ok_or_else(|| {
        AppError::NotFound(format!("doc {}/{}/{} not found", version, lang, slug))
    })?;

    Ok(Json(DocPageResponse {
        slug: page.slug,
        title: page.title,
        lang: page.lang,
        version: page.version,
        html: page.html,
    }))
}

pub async fn get_versioned_doc(
    State(state): State<AppState>,
    Path((version, lang, slug)): Path<(String, String, String)>,
) -> AppResult<Json<DocPageResponse>> {
    let docs = state
        .docs
        .read()
        .map_err(|e| AppError::Internal(format!("docs index lock poisoned: {}", e)))?;
    if !docs.has_version(&version) {
        return Err(AppError::NotFound(format!(
            "unknown docs version '{}'",
            version
        )));
    }
    let page = docs.get(&version, &lang, &slug).ok_or_else(|| {
        AppError::NotFound(format!("doc {}/{}/{} not found", version, lang, slug))
    })?;

    Ok(Json(DocPageResponse {
        slug: page.slug,
        title: page.title,
        lang: page.lang,
        version: page.version,
        html: page.html,
    }))
}

pub async fn list_versions(State(state): State<AppState>) -> AppResult<Json<DocsVersionsResponse>> {
    let docs = state
        .docs
        .read()
        .map_err(|e| AppError::Internal(format!("docs index lock poisoned: {}", e)))?;
    Ok(Json(DocsVersionsResponse {
        versions: docs.versions(),
        latest: docs.default_version().unwrap_or_default(),
    }))
}
