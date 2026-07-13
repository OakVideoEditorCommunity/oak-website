use sea_orm::{Database, DatabaseConnection};

use crate::config::DatabaseConfig;
use crate::error::AppResult;

/// Establishes a connection to the PostgreSQL database.
pub async fn connect(config: &DatabaseConfig) -> AppResult<DatabaseConnection> {
    let db = Database::connect(&config.url).await?;
    Ok(db)
}
