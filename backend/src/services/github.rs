use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

use crate::config::GithubConfig;
use crate::error::{AppError, AppResult};

/// A release asset returned by the GitHub API.
#[derive(Debug, Deserialize)]
pub struct GithubAsset {
    #[allow(dead_code)]
    pub id: u64,
    pub name: String,
    pub size: i64,
    pub browser_download_url: String,
}

/// A GitHub release returned by the GitHub API.
#[derive(Debug, Deserialize)]
pub struct GithubRelease {
    #[allow(dead_code)]
    pub id: u64,
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub prerelease: bool,
    pub published_at: Option<DateTime<FixedOffset>>,
    pub assets: Vec<GithubAsset>,
}

/// Thin client for the GitHub REST API.
pub struct GithubClient {
    client: reqwest::Client,
    owner: String,
    repo: String,
    token: Option<String>,
    api_base: String,
}

impl GithubClient {
    /// Creates a new client from configuration.
    pub fn new(config: &GithubConfig) -> Self {
        let token = config.token.as_ref().and_then(|t| {
            if t.trim().is_empty() {
                None
            } else {
                Some(t.clone())
            }
        });
        Self {
            client: reqwest::Client::new(),
            owner: config.owner.clone(),
            repo: config.repo.clone(),
            token,
            api_base: config.api_base.clone(),
        }
    }

    /// Fetches all releases for the configured repository.
    ///
    /// GitHub paginates releases (30 per page by default), so keep requesting
    /// pages until a short page is returned. Without this, older releases
    /// silently disappear once the repository has more than one page.
    pub async fn fetch_releases(
        &self) -> AppResult<Vec<GithubRelease>> {
        let mut releases = Vec::new();
        let mut page = 1u32;

        loop {
            let url = format!(
                "{}/repos/{}/{}/releases?per_page=100&page={}",
                self.api_base, self.owner, self.repo, page
            );
            let mut req = self.client.get(&url);
            req = req.header("User-Agent", "oak-website-backend");
            req = req.header("Accept", "application/vnd.github+json");
            if let Some(token) = &self.token {
                req = req.header("Authorization", format!("Bearer {}", token));
            }

            let resp = req.send().await.map_err(|e| AppError::External(e.to_string()))?;
            if !resp.status().is_success() {
                return Err(AppError::External(format!(
                    "GitHub API returned {}: {}",
                    resp.status(),
                    resp.text().await.unwrap_or_default()
                )));
            }

            let page_releases: Vec<GithubRelease> = resp
                .json()
                .await
                .map_err(|e| AppError::External(format!("failed to parse GitHub response: {}", e)))?;

            let received = page_releases.len();
            releases.extend(page_releases);

            // A page with fewer than 100 entries is the last one.
            if received < 100 {
                break;
            }
            page += 1;
        }

        Ok(releases)
    }

    /// Starts streaming a release asset from GitHub.
    pub async fn fetch_asset_bytes(
        &self,
        url: &str,
    ) -> AppResult<reqwest::Response> {
        let mut req = self.client.get(url);
        req = req.header("User-Agent", "oak-website-backend");
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }
        let resp = req.send().await.map_err(|e| AppError::External(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AppError::External(format!(
                "GitHub asset returned {}",
                resp.status()
            )));
        }
        Ok(resp)
    }
}

/// Platforms (package families) we distribute, in display order.
pub const SUPPORTED_PLATFORMS: [&str; 8] = [
    "windows",
    "macos",
    "debian",
    "fedora",
    "arch",
    "openkylin-arm",
    "openkylin-x86",
    "appimage",
];

/// Infers the target platform and architecture from an asset filename.
///
/// The platform doubles as the package family used for download labels:
/// `debian`, `fedora`, `arch`, `openkylin-arm`, `openkylin-x86` and `appimage`
/// are all Linux packages but are presented separately, because they are not
/// interchangeable for users.
pub fn infer_platform_arch(filename: &str) -> (String, Option<String>) {
    let lower = filename.to_lowercase();
    let arch = infer_arch(&lower);

    if lower.ends_with(".appimage") {
        ("appimage".to_string(), arch)
    } else if lower.ends_with(".deb") {
        if lower.contains("openkylin") {
            if arch.as_deref() == Some("arm64") {
                ("openkylin-arm".to_string(), Some("arm64".to_string()))
            } else {
                ("openkylin-x86".to_string(), Some("x86_64".to_string()))
            }
        } else {
            ("debian".to_string(), arch)
        }
    } else if lower.ends_with(".rpm") {
        ("fedora".to_string(), arch)
    } else if lower.contains(".pkg.tar") {
        ("arch".to_string(), arch)
    } else if lower.ends_with(".exe") || lower.ends_with(".msi") || lower.ends_with(".zip") {
        ("windows".to_string(), arch)
    } else if lower.ends_with(".dmg") {
        ("macos".to_string(), arch)
    } else if lower.ends_with(".tar.gz") {
        // Legacy generic Linux tarball, kept for older releases.
        ("linux".to_string(), arch)
    } else {
        ("unknown".to_string(), arch)
    }
}

fn infer_arch(lower: &str) -> Option<String> {
    if lower.contains("aarch64") || lower.contains("arm64") {
        Some("arm64".to_string())
    } else if lower.contains("x86_64") || lower.contains("amd64") || lower.contains("x64") {
        Some("x86_64".to_string())
    } else {
        None
    }
}

/// Returns true when an asset belongs to one of the eight packages we ship.
pub fn is_supported_package(filename: &str) -> bool {
    if is_debug_package(filename) {
        return false;
    }
    let (platform, _) = infer_platform_arch(filename);
    SUPPORTED_PLATFORMS.contains(&platform.as_str())
}

/// Maps a concrete package platform to its OS family, so clients that still
/// ask for `platform=linux` keep receiving a matching package.
pub fn platform_family(platform: &str) -> &str {
    match platform {
        "debian" | "fedora" | "arch" | "openkylin-arm" | "openkylin-x86" | "appimage" | "linux" => {
            "linux"
        }
        other => other,
    }
}

/// Returns true when an asset satisfies a requested platform.
///
/// Specific requests (`debian`, `openkylin-arm`, ...) only match that exact
/// package. The generic `linux` request matches any Linux-family package, so
/// older auto-update clients keep working.
pub fn platform_matches(requested: &str, asset_platform: &str) -> bool {
    requested == asset_platform
        || (requested == "linux" && platform_family(asset_platform) == "linux")
}

/// Display preference used when a request only names an OS family (e.g.
/// `platform=linux`). AppImage is the most portable fallback.
pub fn platform_rank(platform: &str) -> usize {
    match platform {
        "appimage" => 0,
        "linux" => 1,
        "debian" => 2,
        "fedora" => 3,
        "arch" => 4,
        "openkylin-x86" => 5,
        "openkylin-arm" => 6,
        _ => 7,
    }
}

/// Returns true for debug symbol packages that should not be distributed,
/// e.g. Arch `-debug-<ver>.pkg.tar.zst` files published alongside releases.
pub fn is_debug_package(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.contains("-debug-") || lower.ends_with(".debug") || lower.contains(".symbols.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    fn test_config(api_base: &str, token: Option<String>) -> GithubConfig {
        GithubConfig {
            owner: "OakVideoEditorCommunity".to_string(),
            repo: "oak".to_string(),
            token,
            api_base: api_base.to_string(),
        }
    }

    #[test]
    fn new_normalizes_empty_token_to_none() {
        let config = test_config("https://api.github.com", Some("   ".to_string()));
        let client = GithubClient::new(&config);
        assert!(client.token.is_none());
        assert_eq!(client.api_base, "https://api.github.com");
    }

    #[test]
    fn new_keeps_non_empty_token() {
        let config = test_config("https://api.github.com", Some("ghp_secret".to_string()));
        let client = GithubClient::new(&config);
        assert_eq!(client.token, Some("ghp_secret".to_string()));
    }

    #[tokio::test]
    async fn fetch_releases_returns_parsed_releases() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({
                "id": 1,
                "tag_name": "v0.1.0",
                "name": "v0.1.0",
                "body": "first release",
                "prerelease": false,
                "published_at": "2024-01-01T00:00:00Z",
                "assets": [{
                    "id": 10,
                    "name": "oak.exe",
                    "size": 1234,
                    "browser_download_url": "https://example.com/oak.exe"
                }]
            })]))
            .mount(&mock_server)
            .await;

        let client = GithubClient::new(&test_config(&mock_server.uri(), None));
        let releases = client.fetch_releases().await.expect("fetch should succeed");
        assert_eq!(releases.len(), 1);
        assert_eq!(releases[0].tag_name, "v0.1.0");
        assert_eq!(releases[0].assets.len(), 1);
    }

    #[tokio::test]
    async fn fetch_releases_follows_pagination() {
        let mock_server = MockServer::start().await;

        let page_one: Vec<serde_json::Value> = (0..100)
            .map(|i| {
                serde_json::json!({
                    "id": i,
                    "tag_name": format!("v0.1.{}", i),
                    "name": format!("v0.1.{}", i),
                    "body": null,
                    "prerelease": false,
                    "published_at": "2024-01-01T00:00:00Z",
                    "assets": []
                })
            })
            .collect();

        Mock::given(method("GET"))
            .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
            .and(query_param("per_page", "100"))
            .and(query_param("page", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(page_one))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
            .and(query_param("per_page", "100"))
            .and(query_param("page", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({
                "id": 100,
                "tag_name": "v0.2.0",
                "name": "v0.2.0",
                "body": null,
                "prerelease": false,
                "published_at": "2024-02-01T00:00:00Z",
                "assets": []
            })]))
            .mount(&mock_server)
            .await;

        let client = GithubClient::new(&test_config(&mock_server.uri(), None));
        let releases = client.fetch_releases().await.expect("fetch should succeed");
        assert_eq!(releases.len(), 101);
        assert_eq!(releases.last().unwrap().tag_name, "v0.2.0");
    }

    #[tokio::test]
    async fn fetch_releases_returns_error_on_non_2xx() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
            .respond_with(ResponseTemplate::new(500).set_body_string("server error"))
            .mount(&mock_server)
            .await;

        let client = GithubClient::new(&test_config(&mock_server.uri(), None));
        let err = client.fetch_releases().await.unwrap_err();
        assert!(matches!(err, AppError::External(_)));
    }

    #[tokio::test]
    async fn fetch_asset_bytes_returns_response_on_success() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/asset"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"binary"))
            .mount(&mock_server)
            .await;

        let client = GithubClient::new(&test_config(&mock_server.uri(), None));
        let resp = client
            .fetch_asset_bytes(&format!("{}/asset", mock_server.uri()))
            .await
            .expect("download should succeed");
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn fetch_releases_sends_authorization_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
            .and(header("authorization", "Bearer ghp_secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::Value::Array(vec![])))
            .mount(&mock_server)
            .await;

        let client = GithubClient::new(&test_config(&mock_server.uri(), Some("ghp_secret".to_string())));
        let releases = client.fetch_releases().await.expect("fetch should succeed");
        assert!(releases.is_empty());
    }

    #[test]
    fn infer_legacy_linux_tar_gz() {
        let (platform, arch) = infer_platform_arch("oak-0.1.0-linux-x86_64.tar.gz");
        assert_eq!(platform, "linux");
        assert_eq!(arch, Some("x86_64".to_string()));
        // Legacy generic tarballs are not one of the eight supported packages.
        assert!(!is_supported_package("oak-0.1.0-linux-x86_64.tar.gz"));
    }

    #[test]
    fn infer_fedora_rpm() {
        let (platform, arch) = infer_platform_arch("oak-editor-0.5.2-1.fc43.x86_64.rpm");
        assert_eq!(platform, "fedora");
        assert_eq!(arch, Some("x86_64".to_string()));
    }

    #[test]
    fn infer_arch_linux_package() {
        let (platform, arch) = infer_platform_arch("oak-editor-0.5.2-1-x86_64.pkg.tar.zst");
        assert_eq!(platform, "arch");
        assert_eq!(arch, Some("x86_64".to_string()));
    }

    #[test]
    fn infer_windows_zip() {
        let (platform, arch) = infer_platform_arch("oak-0.1.0-win.zip");
        assert_eq!(platform, "windows");
        assert_eq!(arch, None);
    }

    #[test]
    fn infer_macos_app() {
        let (platform, arch) = infer_platform_arch("Oak-0.1.0-macOS-aarch64.dmg");
        assert_eq!(platform, "macos");
        assert_eq!(arch, Some("arm64".to_string()));
    }

    #[test]
    fn infers_all_eight_supported_packages() {
        // Real asset names from the v0.5.2-alpha release.
        let cases = [
            ("oak-editor_0.5.2_x64-setup.exe", "windows", Some("x86_64")),
            ("Oak-macOS-arm64.dmg", "macos", Some("arm64")),
            ("oak-editor_0.5.2+debian_amd64.deb", "debian", Some("x86_64")),
            ("oak-editor-0.5.2-1.fc43.x86_64.rpm", "fedora", Some("x86_64")),
            ("oak-editor-0.5.2-1-x86_64.pkg.tar.zst", "arch", Some("x86_64")),
            ("oak-editor_0.5.2+openkylin_arm64.deb", "openkylin-arm", Some("arm64")),
            ("oak-editor_0.5.2+openkylin_amd64.deb", "openkylin-x86", Some("x86_64")),
            ("oak-editor_0.5.2_x86_64.AppImage", "appimage", Some("x86_64")),
        ];

        for (filename, expected_platform, expected_arch) in cases {
            let (platform, arch) = infer_platform_arch(filename);
            assert_eq!(platform, expected_platform, "filename: {}", filename);
            assert_eq!(
                arch.as_deref(),
                expected_arch,
                "arch for filename: {}",
                filename
            );
            assert!(is_supported_package(filename), "filename: {}", filename);
        }
    }

    #[test]
    fn debug_package_is_not_supported() {
        assert!(!is_supported_package(
            "oak-editor-debug-0.5.2-1-x86_64.pkg.tar.zst"
        ));
    }

    #[test]
    fn platform_matching_keeps_linux_alias_but_not_cross_distro_fallbacks() {
        assert!(platform_matches("linux", "appimage"));
        assert!(platform_matches("linux", "debian"));
        assert!(platform_matches("linux", "linux"));
        assert!(platform_matches("debian", "debian"));
        assert!(!platform_matches("debian", "fedora"));
        assert!(!platform_matches("openkylin-arm", "openkylin-x86"));
        assert!(platform_matches("windows", "windows"));
    }

    #[test]
    fn unknown_asset_is_not_supported() {
        assert!(!is_supported_package("README.md"));
        assert!(!is_supported_package("checksums.txt"));
    }

    #[tokio::test]
    async fn fetch_asset_bytes_sends_authorization_header() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/asset"))
            .and(header("authorization", "Bearer ghp_secret"))
            .respond_with(ResponseTemplate::new(200).set_body_bytes(b"binary"))
            .mount(&mock_server)
            .await;

        let client = GithubClient::new(
            &test_config(&mock_server.uri(), Some("ghp_secret".to_string())),
        );
        let resp = client
            .fetch_asset_bytes(&format!("{}/asset", mock_server.uri()))
            .await
            .expect("download should succeed");
        assert_eq!(resp.status(), 200);
    }

    #[test]
    fn infer_completely_unknown() {
        let (platform, arch) = infer_platform_arch("README.md");
        assert_eq!(platform, "unknown");
        assert_eq!(arch, None);
    }

    #[test]
    fn detects_arch_debug_package() {
        // Real asset name from the oak v0.4.0-alpha release.
        assert!(is_debug_package(
            "oak-video-editor-debug-0.4.0_alpha-1-x86_64.pkg.tar.zst"
        ));
    }

    #[test]
    fn detects_symbol_and_debug_suffix_packages() {
        assert!(is_debug_package("oak-0.1.0.symbols.tar.zst"));
        assert!(is_debug_package("oak-0.1.0.debug"));
    }

    #[test]
    fn keeps_regular_packages() {
        assert!(!is_debug_package("oak-video-editor-0.4.0_alpha-1-x86_64.pkg.tar.zst"));
        assert!(!is_debug_package("Oak_Video_Editor-x86_64.AppImage"));
        assert!(!is_debug_package("oak-video-editor_0.4.0-alpha_amd64.deb"));
        assert!(!is_debug_package("oak-video-editor-0.4.0-alpha-Linux.rpm"));
        assert!(!is_debug_package("Oak-Video-Editor-macOS.dmg"));
        assert!(!is_debug_package("Oak-Video-Editor-Windows-x64.exe"));
    }
}

