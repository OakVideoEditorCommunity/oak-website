mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use oak_website_backend::entities::{download_logs, release_assets, releases};
use sea_orm::{ActiveModelTrait, Set};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn list_releases_returns_empty_array() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let response = app
        .oneshot(Request::builder().uri("/api/v1/releases").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["releases"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn latest_release_returns_404_when_empty() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let response = app
        .oneshot(Request::builder().uri("/api/v1/releases/latest").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_release_by_id_works() {
    let (db, _tmp) = common::setup_test_db().await;
    let release_id = Uuid::new_v4();

    let release = releases::ActiveModel {
        id: Set(release_id),
        version: Set("v1.0.0".to_string()),
        tag_name: Set("v1.0.0".to_string()),
        release_notes: Set(None),
        is_prerelease: Set(false),
        published_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    release.insert(&db).await.unwrap();

    let app = common::build_test_app(db).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/releases/{}", release_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], "v1.0.0");
}

#[tokio::test]
async fn download_returns_400_when_asset_not_ready() {
    let (db, _tmp) = common::setup_test_db().await;
    let release_id = Uuid::new_v4();
    let asset_id = Uuid::new_v4();

    let release = releases::ActiveModel {
        id: Set(release_id),
        version: Set("v1.0.0".to_string()),
        tag_name: Set("v1.0.0".to_string()),
        release_notes: Set(None),
        is_prerelease: Set(false),
        published_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    release.insert(&db).await.unwrap();

    let asset = release_assets::ActiveModel {
        id: Set(asset_id),
        release_id: Set(release_id),
        platform: Set("linux".to_string()),
        arch: Set(None),
        filename: Set("test.AppImage".to_string()),
        github_url: Set("https://example.com".to_string()),
        r2_key: Set(None),
        r2_etag: Set(None),
        size_bytes: Set(None),
        sync_status: Set("pending".to_string()),
        synced_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    asset.insert(&db).await.unwrap();

    let app = common::build_test_app(db).await;
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/v1/releases/{}/download?platform=linux",
                    release_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_releases_includes_assets() {
    let (db, _tmp) = common::setup_test_db().await;
    let release_id = Uuid::new_v4();

    let release = releases::ActiveModel {
        id: Set(release_id),
        version: Set("v1.0.0".to_string()),
        tag_name: Set("v1.0.0".to_string()),
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
        platform: Set("linux".to_string()),
        arch: Set(None),
        filename: Set("test.AppImage".to_string()),
        github_url: Set("https://example.com".to_string()),
        r2_key: Set(None),
        r2_etag: Set(None),
        size_bytes: Set(Some(1024)),
        sync_status: Set("pending".to_string()),
        synced_at: Set(None),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    asset.insert(&db).await.unwrap();

    let app = common::build_test_app(db).await;
    let response = app
        .oneshot(Request::builder().uri("/api/v1/releases").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["releases"].as_array().unwrap().len(), 1);
    assert_eq!(json["releases"][0]["assets"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn latest_release_returns_release_with_assets() {
    let (db, _tmp) = common::setup_test_db().await;
    let release_id = Uuid::new_v4();

    let release = releases::ActiveModel {
        id: Set(release_id),
        version: Set("v1.0.0".to_string()),
        tag_name: Set("v1.0.0".to_string()),
        release_notes: Set(None),
        is_prerelease: Set(false),
        published_at: Set(Some(chrono::Utc::now().into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    };
    release.insert(&db).await.unwrap();

    let app = common::build_test_app(db).await;
    let response = app
        .oneshot(Request::builder().uri("/api/v1/releases/latest").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["version"], "v1.0.0");
    assert!(json["assets"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn download_stats_empty_when_no_downloads() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let response = app
        .oneshot(Request::builder().uri("/api/v1/stats/downloads").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total"], 0);
    assert!(json["per_asset"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn download_stats_counts_downloads_per_asset() {
    let (db, _tmp) = common::setup_test_db().await;
    let release_id = Uuid::new_v4();
    let asset_a = Uuid::new_v4();
    let asset_b = Uuid::new_v4();

    releases::ActiveModel {
        id: Set(release_id),
        version: Set("v1.0.0".to_string()),
        tag_name: Set("v1.0.0".to_string()),
        release_notes: Set(None),
        is_prerelease: Set(false),
        published_at: Set(Some(chrono::Utc::now().into())),
        created_at: Set(chrono::Utc::now().into()),
        updated_at: Set(chrono::Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    for (id, filename) in [(asset_a, "a.exe"), (asset_b, "b.AppImage")] {
        release_assets::ActiveModel {
            id: Set(id),
            release_id: Set(release_id),
            platform: Set("windows".to_string()),
            arch: Set(None),
            filename: Set(filename.to_string()),
            github_url: Set("https://example.com".to_string()),
            r2_key: Set(Some(format!("releases/{}/{}", release_id, filename))),
            r2_etag: Set(None),
            size_bytes: Set(Some(1024)),
            sync_status: Set("ready".to_string()),
            synced_at: Set(None),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        }
        .insert(&db)
        .await
        .unwrap();
    }

    // Two downloads of asset A, one of asset B.
    for asset_id in [asset_a, asset_a, asset_b] {
        download_logs::ActiveModel {
            id: Set(Uuid::new_v4()),
            asset_id: Set(asset_id),
            ip: Set(None),
            user_agent: Set(None),
            created_at: Set(chrono::Utc::now().into()),
        }
        .insert(&db)
        .await
        .unwrap();
    }

    let app = common::build_test_app(db).await;
    let response = app
        .oneshot(Request::builder().uri("/api/v1/stats/downloads").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["total"], 3);

    let per_asset = json["per_asset"].as_array().unwrap();
    assert_eq!(per_asset.len(), 2);
    let count_of = |id: Uuid| {
        per_asset
            .iter()
            .find(|e| e["asset_id"] == id.to_string())
            .map(|e| e["count"].as_i64().unwrap())
    };
    assert_eq!(count_of(asset_a), Some(2));
    assert_eq!(count_of(asset_b), Some(1));
}
