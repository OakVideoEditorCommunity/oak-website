use axum::body::Body;
use axum::http::{Request, StatusCode};
use sea_orm::EntityTrait;
use std::time::Duration;
use tower::ServiceExt;
use wiremock::{
    matchers::{method, path, path_regex},
    Mock, MockServer, ResponseTemplate,
};

mod common;
use common::{build_test_app_with_config, setup_test_db};
use oak_website_backend::config::{
    AdminConfig, AppConfig, DatabaseConfig, DocsConfig, GithubConfig, R2Config, ServerConfig,
};

#[tokio::test]
async fn sync_endpoint_requires_authentication() {
    let (db, _tmp) = setup_test_db().await;
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
            endpoint_url: "https://r2.example.com".to_string(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            bucket_name: "bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "secret".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    };

    let app = build_test_app_with_config(db, config).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/releases/sync")
                .header("content-type", "application/json")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn sync_endpoint_syncs_release_and_asset() {
    let (db, _tmp) = setup_test_db().await;

    let github_mock = MockServer::start().await;
    let r2_mock = MockServer::start().await;

    let asset_url = format!("{}/asset", github_mock.uri());
    Mock::given(method("GET"))
        .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({
            "id": 1,
            "tag_name": "v0.1.0",
            "name": "v0.1.0",
            "body": "release notes",
            "prerelease": false,
            "published_at": "2024-01-01T00:00:00Z",
            "assets": [{
                "id": 10,
                "name": "oak.exe",
                "size": 14,
                "browser_download_url": asset_url
            }]
        })]))
        .mount(&github_mock)
        .await;

    Mock::given(method("GET"))
        .and(path("/asset"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", "14")
                .set_body_bytes(b"fake exe bytes"),
        )
        .mount(&github_mock)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex(r"^/test-bucket/releases/[0-9a-fA-F-]+/oak\.exe$"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"etag123\""))
        .mount(&r2_mock)
        .await;

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
            api_base: github_mock.uri(),
        },
        r2: R2Config {
            endpoint_url: r2_mock.uri(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            bucket_name: "test-bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "admin-token".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    };

    let app = build_test_app_with_config(db, config).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/releases/sync")
                .header("content-type", "application/json")
                .header("authorization", "Bearer admin-token")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Allow the async handler to finish before checking the mock server.
    tokio::time::sleep(Duration::from_millis(100)).await;
    github_mock.verify().await;
    r2_mock.verify().await;
}

#[tokio::test]
async fn sync_endpoint_filters_by_tag() {
    let (db, _tmp) = setup_test_db().await;

    let github_mock = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![
            serde_json::json!({
                "id": 1,
                "tag_name": "v0.1.0",
                "name": "v0.1.0",
                "body": null,
                "prerelease": false,
                "published_at": "2024-01-01T00:00:00Z",
                "assets": []
            }),
            serde_json::json!({
                "id": 2,
                "tag_name": "v0.2.0",
                "name": "v0.2.0",
                "body": null,
                "prerelease": false,
                "published_at": "2024-02-01T00:00:00Z",
                "assets": []
            }),
        ]))
        .mount(&github_mock)
        .await;

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
            api_base: github_mock.uri(),
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
            token: "admin-token".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    };

    let app = build_test_app_with_config(db, config).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/releases/sync")
                .header("content-type", "application/json")
                .header("authorization", "Bearer admin-token")
                .body(Body::from(r#"{"tag":"v0.2.0"}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn sync_endpoint_is_idempotent() {
    let (db, _tmp) = setup_test_db().await;

    let github_mock = MockServer::start().await;
    let r2_mock = MockServer::start().await;

    let asset_url = format!("{}/asset", github_mock.uri());
    Mock::given(method("GET"))
        .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({
            "id": 1,
            "tag_name": "v0.1.0",
            "name": "v0.1.0",
            "body": null,
            "prerelease": false,
            "published_at": "2024-01-01T00:00:00Z",
            "assets": [{
                "id": 10,
                "name": "oak.exe",
                "size": 14,
                "browser_download_url": asset_url
            }]
        })]))
        .expect(2)
        .mount(&github_mock)
        .await;

    Mock::given(method("GET"))
        .and(path("/asset"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", "14")
                .set_body_bytes(b"fake exe bytes"),
        )
        .expect(1)
        .mount(&github_mock)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex(r"^/test-bucket/releases/[0-9a-fA-F-]+/oak\.exe$"))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"etag123\""))
        .expect(1)
        .mount(&r2_mock)
        .await;

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
            api_base: github_mock.uri(),
        },
        r2: R2Config {
            endpoint_url: r2_mock.uri(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            bucket_name: "test-bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "admin-token".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    };

    let app = build_test_app_with_config(db, config).await;

    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/admin/releases/sync")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer admin-token")
                    .body(Body::from(r#"{}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
    github_mock.verify().await;
    r2_mock.verify().await;
}

#[tokio::test]
async fn sync_endpoint_skips_debug_packages() {
    let (db, _tmp) = setup_test_db().await;

    let github_mock = MockServer::start().await;
    let r2_mock = MockServer::start().await;

    let asset_url = format!("{}/asset", github_mock.uri());
    Mock::given(method("GET"))
        .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({
            "id": 1,
            "tag_name": "v0.1.0",
            "name": "v0.1.0",
            "body": null,
            "prerelease": false,
            "published_at": "2024-01-01T00:00:00Z",
            "assets": [
                {
                    "id": 10,
                    "name": "oak-video-editor-0.1.0-1-x86_64.pkg.tar.zst",
                    "size": 14,
                    "browser_download_url": asset_url
                },
                {
                    "id": 11,
                    "name": "oak-video-editor-debug-0.1.0-1-x86_64.pkg.tar.zst",
                    "size": 14,
                    "browser_download_url": asset_url
                }
            ]
        })]))
        .mount(&github_mock)
        .await;

    Mock::given(method("GET"))
        .and(path("/asset"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", "14")
                .set_body_bytes(b"fake pkg bytes"),
        )
        .expect(1)
        .mount(&github_mock)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex(
            r"^/test-bucket/releases/[0-9a-fA-F-]+/oak-video-editor-0\.1\.0-1-x86_64\.pkg\.tar\.zst$",
        ))
        .respond_with(ResponseTemplate::new(200).insert_header("ETag", "\"etag123\""))
        .expect(1)
        .mount(&r2_mock)
        .await;

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
            api_base: github_mock.uri(),
        },
        r2: R2Config {
            endpoint_url: r2_mock.uri(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            bucket_name: "test-bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "admin-token".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    };

    let app = build_test_app_with_config(db.clone(), config).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/releases/sync")
                .header("content-type", "application/json")
                .header("authorization", "Bearer admin-token")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    tokio::time::sleep(Duration::from_millis(100)).await;
    github_mock.verify().await;
    r2_mock.verify().await;

    // Only the regular package must be stored; the debug package is skipped.
    let assets = oak_website_backend::entities::release_assets::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(
        assets[0].filename,
        "oak-video-editor-0.1.0-1-x86_64.pkg.tar.zst"
    );
}

#[tokio::test]
async fn sync_endpoint_marks_asset_failed_when_r2_upload_fails() {
    let (db, _tmp) = setup_test_db().await;

    let github_mock = MockServer::start().await;
    let r2_mock = MockServer::start().await;

    let asset_url = format!("{}/asset", github_mock.uri());
    Mock::given(method("GET"))
        .and(path("/repos/OakVideoEditorCommunity/oak/releases"))
        .respond_with(ResponseTemplate::new(200).set_body_json(vec![serde_json::json!({
            "id": 1,
            "tag_name": "v0.1.0",
            "name": "v0.1.0",
            "body": null,
            "prerelease": false,
            "published_at": "2024-01-01T00:00:00Z",
            "assets": [{
                "id": 10,
                "name": "oak.exe",
                "size": 14,
                "browser_download_url": asset_url
            }]
        })]))
        .mount(&github_mock)
        .await;

    Mock::given(method("GET"))
        .and(path("/asset"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-length", "14")
                .set_body_bytes(b"fake exe bytes"),
        )
        .mount(&github_mock)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex(r"^/test-bucket/releases/[0-9a-fA-F-]+/oak\.exe$"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&r2_mock)
        .await;

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
            api_base: github_mock.uri(),
        },
        r2: R2Config {
            endpoint_url: r2_mock.uri(),
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            bucket_name: "test-bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "admin-token".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    };

    let app = build_test_app_with_config(db.clone(), config).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/releases/sync")
                .header("content-type", "application/json")
                .header("authorization", "Bearer admin-token")
                .body(Body::from(r#"{}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let assets = oak_website_backend::entities::release_assets::Entity::find()
        .all(&db)
        .await
        .unwrap();
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].sync_status, "failed");
}
