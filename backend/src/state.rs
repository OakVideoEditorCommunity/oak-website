use aws_sdk_s3::Client as S3Client;
use chrono::{DateTime, FixedOffset};
use sea_orm::DatabaseConnection;
use std::sync::{Arc, Mutex, RwLock};

use crate::config::AppConfig;
use crate::services::docs::DocsIndex;

/// Progress of the background release-sync task (started via the admin API).
#[derive(Debug, Clone, Default)]
pub struct SyncStatus {
    pub running: bool,
    pub last_finished_at: Option<DateTime<FixedOffset>>,
    pub last_result: Option<String>,
}

/// Shared application state passed to all Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
    pub s3: S3Client,
    /// The docs index is protected by a read-write lock so it can be reloaded
    /// in the background when the documentation is updated.
    pub docs: Arc<RwLock<DocsIndex>>,
    /// Release-sync progress. A plain Mutex is enough: it is only ever held
    /// for quick flag updates, never across an await.
    pub sync: Arc<Mutex<SyncStatus>>,
}

impl AppState {
    /// Creates a new application state instance.
    pub fn new(config: AppConfig, db: DatabaseConnection, s3: S3Client, docs: DocsIndex) -> Self {
        Self {
            config: Arc::new(config),
            db,
            s3,
            docs: Arc::new(RwLock::new(docs)),
            sync: Arc::new(Mutex::new(SyncStatus::default())),
        }
    }
}
