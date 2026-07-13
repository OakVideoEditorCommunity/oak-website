use axum::{
    extract::{Path, State},
    Json,
};

use crate::{
    error::{AppError, AppResult},
    models::{DocPageResponse, DocsIndexResponse},
    state::AppState,
};

pub async fn list_docs(State(state): State<AppState>) -> AppResult<Json<DocsIndexResponse>> {
    Ok(Json(DocsIndexResponse {
        zh: state.docs.list("zh"),
        en: state.docs.list("en"),
    }))
}

pub async fn get_doc(
    State(state): State<AppState>,
    Path((lang, slug)): Path<(String, String)>,
) -> AppResult<Json<DocPageResponse>> {
    let page = state
        .docs
        .get(&lang, &slug)
        .ok_or_else(|| AppError::NotFound(format!("doc {}/{} not found", lang, slug)))?;

    Ok(Json(DocPageResponse {
        slug: page.slug,
        title: page.title,
        lang: page.lang,
        html: page.html,
    }))
}
