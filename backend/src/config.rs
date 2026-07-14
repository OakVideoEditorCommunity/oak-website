use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

/// Database connection settings.
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL.
    pub url: String,
}

/// HTTP server binding settings.
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    /// Host address to bind to.
    pub host: String,
    /// Port to listen on.
    pub port: u16,
}

/// GitHub API configuration for fetching releases.
#[derive(Debug, Deserialize, Clone)]
pub struct GithubConfig {
    /// Repository owner (organization or user).
    pub owner: String,
    /// Repository name.
    pub repo: String,
    /// Optional GitHub personal access token for higher rate limits.
    pub token: Option<String>,
    /// Base URL for the GitHub API. Defaults to the public API.
    #[serde(default = "default_github_api_base")]
    pub api_base: String,
}

fn default_github_api_base() -> String {
    "https://api.github.com".to_string()
}

/// Cloudflare R2 (S3-compatible) configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct R2Config {
    /// R2 S3 endpoint URL.
    pub endpoint_url: String,
    /// R2 access key ID.
    pub access_key_id: String,
    /// R2 secret access key.
    pub secret_access_key: String,
    /// R2 bucket that stores release binaries.
    pub bucket_name: String,
    /// Optional public CDN domain for direct downloads.
    pub public_domain: Option<String>,
    /// S3 region identifier (R2 usually uses "auto").
    pub region: String,
}

/// Admin API security settings.
#[derive(Debug, Deserialize, Clone)]
pub struct AdminConfig {
    /// Bearer token required by admin endpoints.
    pub token: String,
}

/// Static documentation settings.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct DocsConfig {
    /// Directory containing pre-built Sphinx HTML docs.
    pub html_dir: String,
    /// Optional Git URL of the documentation repository. When set, the backend
    /// will periodically clone the docs and rebuild the local index.
    #[serde(default)]
    pub git_url: Option<String>,
    /// How often (in hours) to pull docs from GitHub. Defaults to 24.
    #[serde(default = "default_docs_update_interval_hours")]
    pub update_interval_hours: u64,
}

fn default_docs_update_interval_hours() -> u64 {
    24
}

/// Top-level application configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub github: GithubConfig,
    pub r2: R2Config,
    pub admin: AdminConfig,
    pub docs: DocsConfig,
}

impl AppConfig {
    /// Loads configuration from `config/default`, `config/local`, and environment
    /// variables prefixed with `APP__`.
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn set_test_env() {
        env::set_var("APP__DATABASE__URL", "postgres://localhost/oak");
        env::set_var("APP__SERVER__HOST", "127.0.0.1");
        env::set_var("APP__SERVER__PORT", "8080");
        env::set_var("APP__GITHUB__OWNER", "OakVideoEditorCommunity");
        env::set_var("APP__GITHUB__REPO", "oak");
        env::set_var("APP__GITHUB__TOKEN", "ghp_token");
        env::set_var("APP__R2__ENDPOINT_URL", "https://test.r2.cloudflarestorage.com");
        env::set_var("APP__R2__ACCESS_KEY_ID", "key");
        env::set_var("APP__R2__SECRET_ACCESS_KEY", "secret");
        env::set_var("APP__R2__BUCKET_NAME", "bucket");
        env::set_var("APP__R2__REGION", "auto");
        env::set_var("APP__ADMIN__TOKEN", "admin");
        env::set_var("APP__DOCS__HTML_DIR", "/tmp/docs");
    }

    fn unset_test_env() {
        for key in [
            "APP__DATABASE__URL",
            "APP__SERVER__HOST",
            "APP__SERVER__PORT",
            "APP__GITHUB__OWNER",
            "APP__GITHUB__REPO",
            "APP__GITHUB__TOKEN",
            "APP__R2__ENDPOINT_URL",
            "APP__R2__ACCESS_KEY_ID",
            "APP__R2__SECRET_ACCESS_KEY",
            "APP__R2__BUCKET_NAME",
            "APP__R2__REGION",
            "APP__ADMIN__TOKEN",
            "APP__DOCS__HTML_DIR",
        ] {
            env::remove_var(key);
        }
    }

    #[test]
    fn loads_config_from_environment() {
        set_test_env();
        let cfg = AppConfig::new().expect("config should load");
        assert_eq!(cfg.database.url, "postgres://localhost/oak");
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.github.owner, "OakVideoEditorCommunity");
        assert_eq!(cfg.github.repo, "oak");
        assert_eq!(cfg.github.token, Some("ghp_token".to_string()));
        assert_eq!(cfg.github.api_base, "https://api.github.com");
        assert_eq!(cfg.r2.bucket_name, "bucket");
        assert_eq!(cfg.admin.token, "admin");
        assert_eq!(cfg.docs.html_dir, "/tmp/docs");
        unset_test_env();
    }
}
