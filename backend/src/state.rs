use aws_sdk_s3::Client as S3Client;
use sea_orm::DatabaseConnection;
use std::sync::{Arc, RwLock};

use crate::config::AppConfig;
use crate::services::docs::DocsIndex;

/// Shared application state passed to all Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub db: DatabaseConnection,
    pub s3: S3Client,
    /// The docs index is protected by a read-write lock so it can be reloaded
    /// in the background when the documentation is updated.
    pub docs: Arc<RwLock<DocsIndex>>,
}

impl AppState {
    /// Creates a new application state instance.
    pub fn new(config: AppConfig, db: DatabaseConnection, s3: S3Client, docs: DocsIndex) -> Self {
        Self {
            config: Arc::new(config),
            db,
            s3,
            docs: Arc::new(RwLock::new(docs)),
        }
    }
}
