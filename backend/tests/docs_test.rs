mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::fs;
use std::io::Write;
use tower::ServiceExt;

#[tokio::test]
async fn list_docs_returns_empty_when_no_docs() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let response = app
        .oneshot(Request::builder().uri("/api/v1/docs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["zh"].as_array().unwrap().is_empty());
    assert!(json["en"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn get_doc_returns_html() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = tempfile::tempdir().unwrap();
    let zh_dir = html_dir.path().join("zh");
    fs::create_dir_all(&zh_dir).unwrap();
    {
        let mut file = fs::File::create(zh_dir.join("quick_start.html")).unwrap();
        write!(
            file,
            "<!DOCTYPE html><html><head><title>Quick Start</title></head><body>Hello</body></html>"
        )
        .unwrap();
    }

    let app = common::build_test_app_with_docs(
        db,
        zh_dir.parent().unwrap().to_str().unwrap().to_string(),
    )
    .await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs/zh/quick_start")
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
    assert!(json["html"].as_str().unwrap().contains("Hello"));
}
