use oak_website_backend::{
    app::create_app,
    config::{AdminConfig, AppConfig, DatabaseConfig, DocsConfig, GithubConfig, R2Config, ServerConfig},
    db::connect,
    services::{create_s3_client, DocsIndex},
    state::AppState,
};
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm_migration::migrator::MigratorTrait;
use tempfile::TempDir;

/// Creates an in-memory SQLite database and runs migrations.
pub async fn setup_test_db() -> (DatabaseConnection, TempDir) {
    let tmp = TempDir::new().unwrap();

    let db = connect(&DatabaseConfig { url: "sqlite::memory:".to_string() })
        .await
        .expect("failed to connect to sqlite database");

    oak_website_backend::migration::Migrator::up(&db, None)
        .await
        .expect("failed to run migrations");

    (db, tmp)
}

/// Builds a test Axum application using the provided database.
#[allow(dead_code)]
pub async fn build_test_app(db: DatabaseConnection) -> axum::Router {
    let config = AppConfig {
        database: DatabaseConfig {
            url: "sqlite::memory:".to_string(),
        },
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
        },
        github: GithubConfig {
            owner: "OakVideoEditorCommunity".to_string(),
            repo: "oak".to_string(),
            token: None,
            api_base: "https://api.github.com".to_string(),
        },
        r2: R2Config {
            endpoint_url: "https://test.r2.cloudflarestorage.com".to_string(),
            access_key_id: "test".to_string(),
            secret_access_key: "test".to_string(),
            bucket_name: "test-bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "test-token".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir()
                .to_str()
                .unwrap_or("/tmp")
                .to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    };

    let s3 = create_s3_client(&config.r2).expect("failed to create s3 client");
    let docs = DocsIndex::load(&config.docs.html_dir).expect("failed to load docs index");

    let state = AppState::new(config, db, s3, docs);
    create_app(state)
}

/// Builds a test Axum application with an arbitrary configuration.
#[allow(dead_code)]
pub async fn build_test_app_with_docs(db: DatabaseConnection, html_dir: String) -> axum::Router {
    let config = AppConfig {
        database: DatabaseConfig {
            url: "sqlite::memory:".to_string(),
        },
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
        },
        github: GithubConfig {
            owner: "OakVideoEditorCommunity".to_string(),
            repo: "oak".to_string(),
            token: None,
            api_base: "https://api.github.com".to_string(),
        },
        r2: R2Config {
            endpoint_url: "https://test.r2.cloudflarestorage.com".to_string(),
            access_key_id: "test".to_string(),
            secret_access_key: "test".to_string(),
            bucket_name: "test-bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "test-token".to_string(),
        },
        docs: DocsConfig { html_dir, git_url: None, update_interval_hours: 24 },
    };

    let s3 = create_s3_client(&config.r2).expect("failed to create s3 client");
    let docs = DocsIndex::load(&config.docs.html_dir).expect("failed to load docs index");

    let state = AppState::new(config, db, s3, docs);
    create_app(state)
}

/// Helper to truncate all tables between tests.
#[allow(dead_code)]
pub async fn clean_tables(db: &DatabaseConnection) {
    use oak_website_backend::entities::{download_logs, release_assets, releases};
    download_logs::Entity::delete_many()
        .exec(db)
        .await
        .ok();
    release_assets::Entity::delete_many()
        .exec(db)
        .await
        .ok();
    releases::Entity::delete_many()
        .exec(db)
        .await
        .ok();
}

/// Builds a test application from a fully custom configuration.
#[allow(dead_code)]
pub async fn build_test_app_with_config(db: DatabaseConnection, config: AppConfig) -> axum::Router {
    let s3 = create_s3_client(&config.r2).expect("failed to create s3 client");
    let docs = DocsIndex::load(&config.docs.html_dir).expect("failed to load docs index");

    let state = AppState::new(config, db, s3, docs);
    create_app(state)
}
