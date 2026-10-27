use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use chrono::Utc;
use futures::StreamExt;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use std::time::Duration;
use uuid::Uuid;

use crate::{
    entities::{bug_reports, release_assets, releases},
    error::{AppError, AppResult},
    models::{BugReportDto, BugReportListResponse, SyncReleaseRequest, SyncResponse, SyncStatusResponse},
    services::{github::{infer_platform_arch, is_supported_package}, GithubClient, R2Service},
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

/// Starts a release sync in the background and answers 202 immediately. A
/// full sync downloads every asset from GitHub and uploads it to R2, which
/// can take minutes — far longer than any proxy/gateway timeout — so the
/// work runs as a task and progress is polled via `/releases/sync/status`.
pub async fn sync_releases(
    State(state): State<AppState>,
    Json(req): Json<SyncReleaseRequest>,
) -> AppResult<impl IntoResponse> {
    {
        let mut status = state
            .sync
            .lock()
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if status.running {
            return Err(AppError::BadRequest("a sync is already running".to_string()));
        }
        status.running = true;
    }

    let task_state = state.clone();
    tokio::spawn(async move {
        let result = run_sync(&task_state, req).await;
        if let Err(e) = &result {
            tracing::error!("release sync failed: {}", e);
        }
        if let Ok(mut status) = task_state.sync.lock() {
            status.running = false;
            status.last_finished_at = Some(Utc::now().into());
            status.last_result = Some(match result {
                Ok(message) => message,
                Err(e) => format!("failed: {}", e),
            });
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(SyncResponse {
            synced: 0,
            message: "sync started in background".to_string(),
        }),
    ))
}

/// Reports the progress of the background release sync.
pub async fn sync_status(State(state): State<AppState>) -> AppResult<Json<SyncStatusResponse>> {
    let status = state
        .sync
        .lock()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(SyncStatusResponse {
        running: status.running,
        last_finished_at: status.last_finished_at,
        last_result: status.last_result.clone(),
    }))
}

async fn run_sync(state: &AppState, req: SyncReleaseRequest) -> AppResult<String> {
    let github = GithubClient::new(&state.config.github);
    let mut releases_list = github.fetch_releases().await?;

    if let Some(tag) = &req.tag {
        releases_list.retain(|r| &r.tag_name == tag);
    }

    let r2 = R2Service::new(state.s3.clone(), &state.config.r2);
    let mut synced = 0usize;
    let mut skipped = 0usize;
    let mut failed = Vec::new();
    let mut failed_ids = Vec::new();

    for gh in releases_list {
        let existing = releases::Entity::find()
            .filter(releases::Column::TagName.eq(&gh.tag_name))
            .one(&state.db)
            .await?;

        let release_id = match existing {
            Some(r) => {
                // Keep existing rows fresh: release metadata (notes, prerelease
                // flag, publish date) can be edited on GitHub after the first
                // sync. The version column stays untouched because it is the
                // unique display key.
                releases::Entity::update(releases::ActiveModel {
                    id: Set(r.id),
                    release_notes: Set(gh.body.clone()),
                    is_prerelease: Set(gh.prerelease),
                    published_at: Set(gh.published_at.clone().map(|d| d.into())),
                    updated_at: Set(Utc::now().into()),
                    ..Default::default()
                })
                .exec(&state.db)
                .await?;
                r.id
            }
            None => {
                let id = Uuid::new_v4();
                let release = releases::ActiveModel {
                    id: Set(id),
                    version: Set(gh.name.clone().unwrap_or_else(|| gh.tag_name.clone())),
                    tag_name: Set(gh.tag_name.clone()),
                    release_notes: Set(gh.body.clone()),
                    is_prerelease: Set(gh.prerelease),
                    published_at: Set(gh.published_at.clone().map(|d| d.into())),
                    created_at: Set(Utc::now().into()),
                    updated_at: Set(Utc::now().into()),
                };
                release.insert(&state.db).await?;
                id
            }
        };

        for asset in gh.assets {
            // Skip debug symbol packages and anything that is not one of the
            // eight packages we distribute (e.g. checksums or source tarballs).
            if !is_supported_package(&asset.name) {
                skipped += 1;
                continue;
            }

            let (platform, arch) = infer_platform_arch(&asset.name);

            let existing_asset = release_assets::Entity::find()
                .filter(release_assets::Column::ReleaseId.eq(release_id))
                .filter(release_assets::Column::Filename.eq(&asset.name))
                .one(&state.db)
                .await?;

            let asset_id = match existing_asset {
                Some(a) => {
                    // Classification and asset metadata may have changed since
                    // the row was first written (for example after adding a
                    // new platform), so refresh it on every sync.
                    if a.platform != platform
                        || a.arch != arch
                        || a.github_url != asset.browser_download_url
                        || a.size_bytes != Some(asset.size)
                    {
                        release_assets::Entity::update(release_assets::ActiveModel {
                            id: Set(a.id),
                            platform: Set(platform),
                            arch: Set(arch),
                            github_url: Set(asset.browser_download_url.clone()),
                            size_bytes: Set(Some(asset.size)),
                            updated_at: Set(Utc::now().into()),
                            ..Default::default()
                        })
                        .exec(&state.db)
                        .await?;
                    }
                    a.id
                }
                None => {
                    let id = Uuid::new_v4();
                    let model = release_assets::ActiveModel {
                        id: Set(id),
                        release_id: Set(release_id),
                        platform: Set(platform),
                        arch: Set(arch),
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

            match sync_one_asset(&state.db, &github, &r2, asset_id).await {
                Ok(()) => {}
                Err(e) => {
                    tracing::error!("failed to sync asset {}: {}", asset.name, e);
                    failed.push(asset.name.clone());
                    failed_ids.push(asset_id);
                }
            }

            synced += 1;
        }
    }

    // Give assets that failed a second chance after the rest of the release
    // has been processed. A large download that hit a transient network blip
    // usually succeeds once the connection has had time to recover, avoiding
    // a download page that silently lacks one or two platforms.
    if !failed_ids.is_empty() {
        tracing::warn!(
            "{} asset(s) failed; retrying after a short delay",
            failed_ids.len()
        );
        tokio::time::sleep(Duration::from_secs(3)).await;

        for asset_id in failed_ids {
            let Some(asset) = release_assets::Entity::find_by_id(asset_id)
                .one(&state.db)
                .await?
            else {
                continue;
            };

            match sync_one_asset(&state.db, &github, &r2, asset_id).await {
                Ok(()) => {
                    failed.retain(|name| name != &asset.filename);
                }
                Err(e) => {
                    tracing::error!(
                        "asset {} still failed after retry: {}",
                        asset.filename,
                        e
                    );
                }
            }
        }
    }

    if failed.is_empty() {
        Ok(format!(
            "synced {} assets ({} unsupported assets skipped)",
            synced, skipped
        ))
    } else {
        Err(AppError::External(format!(
            "synced {} assets, {} failed: {}",
            synced - failed.len(),
            failed.len(),
            failed.join(", ")
        )))
    }
}

/// Brings a single stored asset to the `ready` state, retrying downloads and
/// uploads. Ready assets with an R2 key are left untouched, making repeated
/// syncs cheap and idempotent.
async fn sync_one_asset(
    db: &sea_orm::DatabaseConnection,
    github: &GithubClient,
    r2: &R2Service,
    asset_id: Uuid,
) -> AppResult<()> {
    let current = release_assets::Entity::find_by_id(asset_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::Internal("asset disappeared".to_string()))?;

    // "syncing" must be retried too: if the backend restarts mid-sync, the
    // asset would otherwise stay in that state forever and never reappear on
    // the download page. A missing R2 key is also treated as unsynced.
    if current.sync_status == "ready" && current.r2_key.is_some() {
        return Ok(());
    }

    release_assets::Entity::update(release_assets::ActiveModel {
        id: Set(asset_id),
        sync_status: Set("syncing".to_string()),
        updated_at: Set(Utc::now().into()),
        ..Default::default()
    })
    .exec(db)
    .await?;

    match sync_asset_with_retry(github, r2, &current).await {
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
            .exec(db)
            .await?;
            Ok(())
        }
        Err(e) => {
            release_assets::Entity::update(release_assets::ActiveModel {
                id: Set(asset_id),
                sync_status: Set("failed".to_string()),
                updated_at: Set(Utc::now().into()),
                ..Default::default()
            })
            .exec(db)
            .await?;
            Err(e)
        }
    }
}

/// Retries a full asset download + upload a few times. Large GitHub downloads
/// and R2 uploads can fail transiently; without this the asset is marked
/// "failed" and silently disappears from the download page until the next
/// manual sync.
async fn sync_asset_with_retry(
    github: &GithubClient,
    r2: &R2Service,
    asset: &release_assets::Model,
) -> AppResult<(String, String)> {
    const MAX_ATTEMPTS: usize = 3;
    let mut last_error = None;

    for attempt in 1..=MAX_ATTEMPTS {
        match sync_asset_to_r2(github, r2, asset).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!(
                    "asset {} attempt {}/{} failed: {}",
                    asset.filename,
                    attempt,
                    MAX_ATTEMPTS,
                    e
                );
                last_error = Some(e);
                if attempt < MAX_ATTEMPTS {
                    tokio::time::sleep(Duration::from_millis(500 * attempt as u64)).await;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| AppError::External("asset sync failed".to_string())))
}

async fn sync_asset_to_r2(
    github: &GithubClient,
    r2: &R2Service,
    asset: &release_assets::Model,
) -> AppResult<(String, String)> {
    let key = format!("releases/{}/{}", asset.release_id, asset.filename);
    let resp = github.fetch_asset_bytes(&asset.github_url).await?;
    let response_length = resp.content_length().unwrap_or(0) as i64;

    let temp_path = std::env::temp_dir().join(format!(
        "oak-asset-{}-{}",
        asset.id,
        Uuid::new_v4()
    ));

    let result = async {
        let downloaded = {
            let mut file = tokio::fs::File::create(&temp_path)
                .await
                .map_err(|e| AppError::Internal(format!("create temp file: {}", e)))?;
            let mut stream = resp.bytes_stream();
            let mut downloaded = 0i64;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk
                    .map_err(|e| AppError::External(format!("download chunk: {}", e)))?;
                tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                    .await
                    .map_err(|e| AppError::Internal(format!("write temp file: {}", e)))?;
                downloaded += chunk.len() as i64;
            }
            tokio::io::AsyncWriteExt::flush(&mut file)
                .await
                .map_err(|e| AppError::Internal(format!("flush temp file: {}", e)))?;
            tokio::io::AsyncWriteExt::shutdown(&mut file)
                .await
                .map_err(|e| AppError::Internal(format!("close temp file: {}", e)))?;
            downloaded
        };

        // Guard against truncated downloads being uploaded as "ready" files.
        // GitHub reports the asset size both in the API payload and (usually)
        // as Content-Length on the redirect target; use whichever is available.
        let expected_length = asset
            .size_bytes
            .filter(|size| *size > 0)
            .or_else(|| (response_length > 0).then_some(response_length));
        if let Some(expected) = expected_length {
            if downloaded != expected {
                return Err(AppError::External(format!(
                    "downloaded {} bytes, expected {} for {}",
                    downloaded, expected, asset.filename
                )));
            }
        }

        let body = aws_sdk_s3::primitives::ByteStream::from_path(&temp_path)
            .await
            .map_err(|e| AppError::Internal(format!("read temp file: {}", e)))?;

        let etag = r2
            .upload_streaming(&key, downloaded, body)
            .await?
            .unwrap_or_default();

        Ok((key, etag))
    }
    .await;

    // Always clean up, including when the download or upload failed.
    let _ = tokio::fs::remove_file(&temp_path).await;

    result
}
