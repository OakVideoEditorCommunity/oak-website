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
    pub html: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocsIndexResponse {
    pub zh: Vec<DocPageSummary>,
    pub en: Vec<DocPageSummary>,
}
