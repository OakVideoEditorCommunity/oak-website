use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder, Set};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    entities::{download_logs, release_assets, releases},
    error::{AppError, AppResult},
    models::{DownloadQuery, ReleaseAssetDto, ReleaseDto, ReleaseListResponse},
    services::R2Service,
    state::AppState,
};

fn to_dto(release: releases::Model, assets: Vec<release_assets::Model>) -> ReleaseDto {
    ReleaseDto {
        id: release.id,
        version: release.version,
        tag_name: release.tag_name,
        release_notes: release.release_notes,
        is_prerelease: release.is_prerelease,
        published_at: release.published_at.map(|d| d.into()),
        assets: assets
            .into_iter()
            .map(|a| ReleaseAssetDto {
                id: a.id,
                platform: a.platform,
                arch: a.arch,
                filename: a.filename,
                size_bytes: a.size_bytes,
                sync_status: a.sync_status,
                synced_at: a.synced_at.map(|d| d.into()),
            })
            .collect(),
    }
}

pub async fn list_releases(State(state): State<AppState>) -> AppResult<Json<ReleaseListResponse>> {
    let releases = releases::Entity::find()
        .order_by_desc(releases::Column::PublishedAt)
        .all(&state.db)
        .await?;

    let mut dtos = Vec::with_capacity(releases.len());
    for release in releases {
        let assets = release.find_related(release_assets::Entity).all(&state.db).await?;
        dtos.push(to_dto(release, assets));
    }

    Ok(Json(ReleaseListResponse { releases: dtos }))
}

pub async fn latest_release(State(state): State<AppState>) -> AppResult<Json<ReleaseDto>> {
    let release = releases::Entity::find()
        .order_by_desc(releases::Column::PublishedAt)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("no release found".to_string()))?;

    let assets = release.find_related(release_assets::Entity).all(&state.db).await?;
    Ok(Json(to_dto(release, assets)))
}

pub async fn get_release(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ReleaseDto>> {
    let release = releases::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("release {} not found", id)))?;

    let assets = release.find_related(release_assets::Entity).all(&state.db).await?;
    Ok(Json(to_dto(release, assets)))
}

pub async fn download_release(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<DownloadQuery>,
    headers: axum::http::HeaderMap,
) -> AppResult<impl IntoResponse> {
    let platform = query.platform.unwrap_or_else(|| "unknown".to_string());
    let arch = query.arch;

    let assets = release_assets::Entity::find()
        .filter(release_assets::Column::ReleaseId.eq(id))
        .all(&state.db)
        .await?;

    let asset = assets
        .into_iter()
        .filter(|a| a.platform == platform)
        .filter(|a| {
            if let Some(ref wanted) = arch {
                a.arch.as_ref() == Some(wanted)
            } else {
                true
            }
        })
        .next()
        .ok_or_else(|| AppError::NotFound(format!("no asset for platform {} arch {:?}", platform, arch)))?;

    if asset.sync_status != "ready" || asset.r2_key.is_none() {
        return Err(AppError::BadRequest("asset not ready yet".to_string()));
    }

    let r2_key = asset.r2_key.unwrap();

    let r2 = R2Service::new(state.s3.clone(), &state.config.r2);
    let presigned = r2
        .generate_presigned_url(&r2_key, Duration::from_secs(300))
        .await?;

    let ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("x-real-ip").and_then(|v| v.to_str().ok()))
        .map(|s| s.to_string());
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let log = download_logs::ActiveModel {
        id: Set(Uuid::new_v4()),
        asset_id: Set(asset.id),
        ip: Set(ip),
        user_agent: Set(user_agent),
        created_at: Set(chrono::Utc::now().into()),
    };

    if let Err(e) = log.insert(&state.db).await {
        tracing::error!("failed to record download log: {}", e);
    }

    Ok(Redirect::temporary(&presigned))
}
