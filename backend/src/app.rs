use std::net::SocketAddr;

use axum::{
    http::{header, Method},
    Router,
};
use sea_orm_migration::migrator::MigratorTrait;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    trace::{DefaultMakeSpan, TraceLayer},
};

use crate::{
    config::AppConfig,
    db::connect,
    routes::build,
    services::{create_s3_client, DocsIndex},
    state::AppState,
};

/// Builds the full Axum application including API routes and middleware layers.
pub fn create_app(state: AppState) -> Router {
    // Mirror the request Origin header instead of hard-coding origins. This makes
    // local development (different ports) and arbitrary production frontends work
    // without redeploying the backend, while still requiring a cross-origin request.
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    Router::new()
        .merge(build(state.clone()))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(false)),
        )
        .with_state(state)
}

/// Bootstraps the database, S3 client, docs index, and application router.
///
/// Returns the router and the socket address it should bind to. The caller is
/// responsible for creating the TCP listener and serving the app.
pub async fn bootstrap(config: AppConfig) -> anyhow::Result<(Router, SocketAddr)> {
    let db = connect(&config.database).await?;
    crate::migration::Migrator::up(&db, None).await?;

    let s3 = create_s3_client(&config.r2)?;
    let docs = DocsIndex::load(&config.docs.html_dir)?;

    let state = AppState::new(config.clone(), db, s3, docs);
    let app = create_app(state);

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid server address: {}", e))?;

    Ok((app, addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AdminConfig, DatabaseConfig, DocsConfig, GithubConfig, R2Config, ServerConfig,
    };

    fn test_config() -> AppConfig {
        AppConfig {
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
                endpoint_url: "https://r2.example.com".to_string(),
                access_key_id: "key".to_string(),
                secret_access_key: "secret".to_string(),
                bucket_name: "bucket".to_string(),
                public_domain: None,
                region: "auto".to_string(),
            },
            admin: AdminConfig {
                token: "token".to_string(),
            },
            docs: DocsConfig {
                html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            },
        }
    }

    #[tokio::test]
    async fn bootstrap_creates_app_and_address() {
        let config = test_config();
        let (app, addr) = bootstrap(config).await.expect("bootstrap should succeed");
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        // The returned router should have at least the health route configured.
        // We cannot easily query routes, so just ensure it was built.
        let _ = app;
    }
}
