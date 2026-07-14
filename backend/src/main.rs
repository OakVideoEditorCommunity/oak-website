use std::time::Duration;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use oak_website_backend::{app::bootstrap, config::AppConfig, services::docs::pull_docs_from_git};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oak_website_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::new().map_err(|e| anyhow::anyhow!("config error: {}", e))?;

    let (app, addr, state) = bootstrap(config.clone()).await?;

    spawn_docs_sync_task(&config, state.docs.clone());

    tracing::info!("listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Spawns a background task that periodically pulls the latest documentation
/// from the configured Git repository and reloads the in-memory index.
fn spawn_docs_sync_task(
    config: &AppConfig,
    docs: std::sync::Arc<std::sync::RwLock<oak_website_backend::services::docs::DocsIndex>>,
) {
    let Some(git_url) = config.docs.git_url.clone() else {
        tracing::info!("no docs git url configured; skipping background docs sync");
        return;
    };

    let html_dir = config.docs.html_dir.clone();
    let interval_hours = config.docs.update_interval_hours.max(1);

    tokio::spawn(async move {
        let interval = Duration::from_secs(interval_hours * 3600);

        loop {
            if let Err(e) = pull_docs_from_git(&git_url, &html_dir, &docs).await {
                tracing::error!("background docs sync failed: {}", e);
            }
            tokio::time::sleep(interval).await;
        }
    });
}
