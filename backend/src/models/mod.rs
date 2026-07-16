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
pub struct SyncReleaseRequest {
    pub tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub synced: usize,
    pub message: String,
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
