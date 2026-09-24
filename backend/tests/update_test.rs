mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use oak_website_backend::entities::{release_assets, releases};
use sea_orm::{ActiveModelTrait, Set};
use tower::ServiceExt;
use uuid::Uuid;

fn release_model(version: &str, is_prerelease: bool, published_at: chrono::DateTime<chrono::Utc>) -> releases::ActiveModel {
    releases::ActiveModel {
        id: Set(Uuid::new_v4()),
        version: Set(version.to_string()),
        tag_name: Set(version.to_string()),
        release_notes: Set(Some(format!("notes for {}", version))),
        is_prerelease: Set(is_prerelease),
        published_at: Set(Some(published_at.into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    }
}

#[tokio::test]
async fn latest_update_returns_404_when_empty() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let response = app
        .oneshot(Request::builder().uri("/api/v1/update/latest").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn latest_update_prefers_stable_over_newer_prerelease() {
    let (db, _tmp) = common::setup_test_db().await;

    let older = chrono::Utc::now();
    let newer = older + chrono::Duration::days(1);
    release_model("v1.0.0", false, older).insert(&db).await.unwrap();
    release_model("v2.0.0-rc.1", true, newer).insert(&db).await.unwrap();

    let app = common::build_test_app(db).await;
    let response = app
        .oneshot(Request::builder().uri("/api/v1/update/latest").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], "v1.0.0");
    assert_eq!(json["is_prerelease"], false);
    assert_eq!(json["notes"], "notes for v1.0.0");
    assert!(json["published_at"].is_string());
    assert!(json["download_url"].is_null());
}

#[tokio::test]
async fn latest_update_falls_back_to_prerelease_when_no_stable() {
    let (db, _tmp) = common::setup_test_db().await;

    release_model("v0.1.0-alpha.1", true, chrono::Utc::now())
        .insert(&db)
        .await
        .unwrap();

    let app = common::build_test_app(db).await;
    let response = app
        .oneshot(Request::builder().uri("/api/v1/update/latest").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], "v0.1.0-alpha.1");
    assert_eq!(json["is_prerelease"], true);
}

#[tokio::test]
async fn latest_update_returns_download_url_for_ready_platform_asset() {
    let (db, _tmp) = common::setup_test_db().await;

    let release = release_model("v1.0.0", false, chrono::Utc::now());
    let release_id = release.id.clone().unwrap();
    release.insert(&db).await.unwrap();

    let asset_id = Uuid::new_v4();
    let asset = release_assets::ActiveModel {
        id: Set(asset_id),
        release_id: Set(release_id),
        platform: Set("windows".to_string()),
        arch: Set(Some("x86_64".to_string())),
        filename: Set("oak-setup.exe".to_string()),
        github_url: Set("https://example.com".to_string()),
        r2_key: Set(Some("releases/v1.0.0/oak-setup.exe".to_string())),
        r2_etag: Set(None),
        size_bytes: Set(Some(2048)),
        sync_status: Set("ready".to_string()),
        synced_at: Set(Some(chrono::Utc::now().into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    asset.insert(&db).await.unwrap();

    // A not-yet-ready asset must not produce a download URL.
    let pending_asset = release_assets::ActiveModel {
        id: Set(Uuid::new_v4()),
        release_id: Set(release_id),
        platform: Set("linux".to_string()),
        arch: Set(None),
        filename: Set("oak.AppImage".to_string()),
        github_url: Set("https://example.com".to_string()),
        r2_key: Set(None),
        r2_etag: Set(None),
        size_bytes: Set(None),
        sync_status: Set("pending".to_string()),
        synced_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    pending_asset.insert(&db).await.unwrap();

    let app = common::build_test_app(db).await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/update/latest?platform=windows&arch=x86_64")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        json["download_url"],
        format!("/api/v1/releases/{}/download?asset_id={}", release_id, asset_id)
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/update/latest?platform=linux")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], "v1.0.0");
    assert!(json["download_url"].is_null());
}
