use axum::{extract::State, Json};
use chrono::Utc;
use futures::StreamExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    entities::{bug_reports, release_assets, releases},
    error::{AppError, AppResult},
    models::{BugReportDto, BugReportListResponse, SyncReleaseRequest, SyncResponse},
    services::{github::{infer_platform_arch, is_debug_package}, GithubClient, R2Service},
    state::AppState,
};

/// Lists stored bug reports, newest first. Attachment keys are turned into
/// short-lived presigned URLs so admins can open them straight from the list.
pub async fn list_bug_reports(State(state): State<AppState>) -> AppResult<Json<BugReportListResponse>> {
    let reports = bug_reports::Entity::find()
        .order_by_desc(bug_reports::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let r2 = R2Service::new(state.s3.clone(), &state.config.r2);
    let mut dtos = Vec::with_capacity(reports.len());
    for report in reports {
        let mut screenshot_url = None;
        let mut log_url = None;
        if let Some(key) = &report.screenshot_key {
            screenshot_url = r2.generate_presigned_url(key, Duration::from_secs(3600)).await.ok();
        }
        if let Some(key) = &report.log_key {
            log_url = r2.generate_presigned_url(key, Duration::from_secs(3600)).await.ok();
        }
        dtos.push(BugReportDto {
            id: report.id,
            title: report.title,
            app_version: report.app_version,
            content: report.content,
            email: report.email,
            screenshot_filename: report.screenshot_filename,
            screenshot_url,
            log_filename: report.log_filename,
            log_url,
            created_at: report.created_at.into(),
        });
    }

    Ok(Json(BugReportListResponse { reports: dtos }))
}

pub async fn sync_releases(
    State(state): State<AppState>,
    Json(req): Json<SyncReleaseRequest>,
) -> AppResult<Json<SyncResponse>> {
    let github = GithubClient::new(&state.config.github);
    let mut releases_list = github.fetch_releases().await?;

    if let Some(tag) = &req.tag {
        releases_list.retain(|r| &r.tag_name == tag);
    }

    let r2 = R2Service::new(state.s3.clone(), &state.config.r2);
    let mut synced = 0usize;

    for gh in releases_list {
        let existing = releases::Entity::find()
            .filter(releases::Column::TagName.eq(&gh.tag_name))
            .one(&state.db)
            .await?;

        let release_id = match existing {
            Some(r) => r.id,
            None => {
                let id = Uuid::new_v4();
                let release = releases::ActiveModel {
                    id: Set(id),
                    version: Set(gh.name.unwrap_or_else(|| gh.tag_name.clone())),
                    tag_name: Set(gh.tag_name.clone()),
                    release_notes: Set(gh.body),
                    is_prerelease: Set(gh.prerelease),
                    published_at: Set(gh.published_at.map(|d| d.into())),
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                };
                release.insert(&state.db).await?;
                id
            }
        };

        for asset in gh.assets {
            // Skip debug symbol packages; they are not end-user downloads.
            if is_debug_package(&asset.name) {
                continue;
            }

            let existing_asset = release_assets::Entity::find()
                .filter(release_assets::Column::ReleaseId.eq(release_id))
                .filter(release_assets::Column::Filename.eq(&asset.name))
                .one(&state.db)
                .await?;

            let (platform, arch) = infer_platform_arch(&asset.name);

            let asset_id = match existing_asset {
                Some(a) => a.id,
                None => {
                    let id = Uuid::new_v4();
                    let model = release_assets::ActiveModel {
                        id: Set(id),
                        release_id: Set(release_id),
                        platform: Set(platform.clone()),
                        arch: Set(arch.clone()),
                        filename: Set(asset.name.clone()),
                        github_url: Set(asset.browser_download_url.clone()),
                        r2_key: Set(None),
                        r2_etag: Set(None),
                        size_bytes: Set(Some(asset.size)),
                        sync_status: Set("pending".to_string()),
                        synced_at: Set(None),
                        created_at: Set(Utc::now().into()),
                        updated_at: Set(Utc::now().into()),
                    };
                    model.insert(&state.db).await?;
                    id
                }
            };

            let current = release_assets::Entity::find_by_id(asset_id)
                .one(&state.db)
                .await?
                .ok_or_else(|| AppError::Internal("asset disappeared".to_string()))?;

            // Retry everything that is not fully synced. "syncing" must be
            // retried too: if the backend restarts mid-sync, the asset would
            // otherwise stay in that state forever and never reappear on the
            // download page.
            if current.sync_status != "ready" {
                release_assets::Entity::update(release_assets::ActiveModel {
                    id: Set(asset_id),
                    sync_status: Set("syncing".to_string()),
                    updated_at: Set(Utc::now().into()),
                    ..Default::default()
                })
                .exec(&state.db)
                .await?;

                match sync_asset_to_r2(&github,&r2, &current).await {
                    Ok((r2_key, etag)) => {
                        release_assets::Entity::update(release_assets::ActiveModel {
                            id: Set(asset_id),
                            r2_key: Set(Some(r2_key)),
                            r2_etag: Set(Some(etag)),
                            sync_status: Set("ready".to_string()),
                            synced_at: Set(Some(Utc::now().into())),
                            updated_at: Set(Utc::now().into()),
                            ..Default::default()
                        })
                        .exec(&state.db)
                        .await?;
                    }
                    Err(e) => {
                        tracing::error!("failed to sync asset {}: {}", asset.name, e);
                        release_assets::Entity::update(release_assets::ActiveModel {
                            id: Set(asset_id),
                            sync_status: Set("failed".to_string()),
                            updated_at: Set(Utc::now().into()),
                            ..Default::default()
                        })
                        .exec(&state.db)
                        .await?;
                    }
                }
            }

            synced += 1;
        }
    }

    Ok(Json(SyncResponse {
        synced,
        message: format!("synced {} assets", synced),
    }))
}

async fn sync_asset_to_r2(
    github: &GithubClient,
    r2: &R2Service,
    asset: &release_assets::Model,
) -> AppResult<(String, String)> {
    let key = format!("releases/{}/{}", asset.release_id, asset.filename);
    let resp = github.fetch_asset_bytes(&asset.github_url).await?;
    let content_length = resp.content_length().unwrap_or(0) as i64;

    let temp_path = std::env::temp_dir().join(format!("oak-asset-{}", asset.id));
    {
        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| AppError::Internal(format!("create temp file: {}", e)))?;
        let mut stream = resp.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::External(format!("download chunk: {}", e)))?;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                .await
                .map_err(|e| AppError::Internal(format!("write temp file: {}", e)))?;
        }
    }

    let body = aws_sdk_s3::primitives::ByteStream::from_path(&temp_path)
        .await
        .map_err(|e| AppError::Internal(format!("read temp file: {}", e)))?;

    let etag = r2.upload_streaming(&key, content_length, body).await?.unwrap_or_default();

    // Clean up the temporary file after upload.
    let _ = tokio::fs::remove_file(&temp_path).await;

    Ok((key, etag))
}
