use axum::body::Body;
use axum::http::{Request, StatusCode};
use sea_orm::{ActiveModelTrait, Set};
use tower::ServiceExt;
use uuid::Uuid;

mod common;
use common::{build_test_app_with_config, setup_test_db};
use oak_website_backend::config::{
    AdminConfig, AppConfig, DatabaseConfig, DocsConfig, GithubConfig, R2Config, ServerConfig,
};
use oak_website_backend::entities::{release_assets, releases};

async fn insert_ready_release(db: &sea_orm::DatabaseConnection) -> (Uuid, Uuid) {
    let release_id = Uuid::new_v4();
    let release = releases::ActiveModel {
        id: Set(release_id),
        version: Set("v0.1.0".to_string()),
        tag_name: Set("v0.1.0".to_string()),
        release_notes: Set(None),
        is_prerelease: Set(false),
        published_at: Set(Some(chrono::Utc::now().into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    release.insert(db).await.unwrap();

    let asset_id = Uuid::new_v4();
    let r2_key = format!("releases/{}/oak.exe", release_id);
    let asset = release_assets::ActiveModel {
        id: Set(asset_id),
        release_id: Set(release_id),
        platform: Set("windows".to_string()),
        arch: Set(Some("x86_64".to_string())),
        filename: Set("oak.exe".to_string()),
        github_url: Set("https://github.com/asset".to_string()),
        r2_key: Set(Some(r2_key)),
        r2_etag: Set(Some("etag".to_string())),
        size_bytes: Set(Some(1234)),
        sync_status: Set("ready".to_string()),
        synced_at: Set(Some(chrono::Utc::now().into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    asset.insert(db).await.unwrap();

    (release_id, asset_id)
}

fn test_config(endpoint_url: String) -> AppConfig {
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
            endpoint_url,
            access_key_id: "key".to_string(),
            secret_access_key: "secret".to_string(),
            bucket_name: "bucket".to_string(),
            public_domain: None,
            region: "auto".to_string(),
        },
        admin: AdminConfig {
            token: "admin".to_string(),
        },
        docs: DocsConfig {
            html_dir: std::env::temp_dir().to_str().unwrap().to_string(),
            git_url: None,
            update_interval_hours: 24,
        },
    }
}

#[tokio::test]
async fn download_redirects_to_presigned_url() {
    let (db, _tmp) = setup_test_db().await;
    let (release_id, _asset_id) = insert_ready_release(&db).await;
    let config = test_config("https://r2.example.com".to_string());
    let app = build_test_app_with_config(db, config.clone()).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/releases/{}/download?platform=windows&arch=x86_64",
                    release_id
                ))
                .header("user-agent", "test-agent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = response
        .headers()
        .get("location")
        .expect("location header should exist")
        .to_str()
        .unwrap();
    assert!(location.starts_with(&config.r2.endpoint_url));
    assert!(location.contains("releases/"));
}

#[tokio::test]
async fn download_filters_by_arch() {
    let (db, _tmp) = setup_test_db().await;
    let (release_id, _asset_id) = insert_ready_release(&db).await;
    let config = test_config("https://r2.example.com".to_string());
    let app = build_test_app_with_config(db, config).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/releases/{}/download?platform=windows&arch=arm64",
                    release_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn download_rejects_not_ready_asset() {
    let (db, _tmp) = setup_test_db().await;
    let release_id = Uuid::new_v4();
    let release = releases::ActiveModel {
        id: Set(release_id),
        version: Set("v0.1.0".to_string()),
        tag_name: Set("v0.1.0".to_string()),
        release_notes: Set(None),
        is_prerelease: Set(false),
        published_at: Set(Some(chrono::Utc::now().into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    release.insert(&db).await.unwrap();

    let asset = release_assets::ActiveModel {
        id: Set(Uuid::new_v4()),
        release_id: Set(release_id),
        platform: Set("windows".to_string()),
        arch: Set(Some("x86_64".to_string())),
        filename: Set("oak.exe".to_string()),
        github_url: Set("https://github.com/asset".to_string()),
        r2_key: Set(None),
        r2_etag: Set(None),
        size_bytes: Set(Some(1234)),
        sync_status: Set("pending".to_string()),
        synced_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    asset.insert(&db).await.unwrap();

    let config = test_config("https://r2.example.com".to_string());
    let app = build_test_app_with_config(db, config).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!(
                    "/api/v1/releases/{}/download?platform=windows",
                    release_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
