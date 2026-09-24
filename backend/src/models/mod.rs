use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseAssetDto {
    pub id: Uuid,
    pub platform: String,
    pub arch: Option<String>,
    pub filename: String,
    pub size_bytes: Option<i64>,
    pub sync_status: String,
    pub synced_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseDto {
    pub id: Uuid,
    pub version: String,
    pub tag_name: String,
    pub release_notes: Option<String>,
    pub is_prerelease: bool,
    pub published_at: Option<DateTime<FixedOffset>>,
    pub assets: Vec<ReleaseAssetDto>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseListResponse {
    pub releases: Vec<ReleaseDto>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadQuery {
    pub platform: Option<String>,
    pub arch: Option<String>,
    /// Exact asset to download. Takes precedence over platform/arch matching,
    /// which is ambiguous when one platform has multiple packages (e.g. Linux
    /// ships deb, rpm, AppImage and pkg.tar.zst).
    pub asset_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuery {
    pub platform: Option<String>,
    pub arch: Option<String>,
}

/// Payload for the auto-update check. `download_url` is a site-relative path
/// that 302-redirects to the real file, and is only present when a `platform`
/// was requested and a matching ready asset exists.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfoResponse {
    pub version: String,
    pub tag_name: String,
    pub notes: Option<String>,
    pub is_prerelease: bool,
    pub published_at: Option<DateTime<FixedOffset>>,
    pub download_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyncReleaseRequest {
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub synced: usize,
    pub message: String,
}

/// Response after a successful bug-report submission.
#[derive(Debug, Serialize, Deserialize)]
pub struct BugReportSubmitResponse {
    pub id: Uuid,
    pub message: String,
}

/// Admin view of a bug report. Attachments are exposed as time-limited
/// presigned R2 URLs (None when the report has no such attachment or signing
/// failed).
#[derive(Debug, Serialize, Deserialize)]
pub struct BugReportDto {
    pub id: Uuid,
    pub title: String,
    pub app_version: String,
    pub content: String,
    pub email: Option<String>,
    pub screenshot_filename: Option<String>,
    pub screenshot_url: Option<String>,
    pub log_filename: Option<String>,
    pub log_url: Option<String>,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BugReportListResponse {
    pub reports: Vec<BugReportDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocPageSummary {
    pub slug: String,
    pub title: String,
    pub lang: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocPageResponse {
    pub slug: String,
    pub title: String,
    pub lang: String,
    pub version: String,
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocsIndexResponse {
    /// The documentation version these TOCs belong to.
    pub version: String,
    pub zh: Vec<DocPageSummary>,
    pub en: Vec<DocPageSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocsVersionsResponse {
    pub versions: Vec<String>,
    pub latest: String,
}

#[derive(Debug, Deserialize)]
pub struct DocsIndexQuery {
    /// Documentation version to list. Defaults to the index's default version.
    pub version: Option<String>,
}
