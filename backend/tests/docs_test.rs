mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::fs;
use std::io::Write;
use std::path::Path;
use tower::ServiceExt;

fn write_test_doc(dir: &Path, slug: &str, title: &str) {
    let mut file = fs::File::create(dir.join(format!("{}.html", slug))).unwrap();
    write!(
        file,
        "<!DOCTYPE html><html><head><title>{}</title></head><body>{}</body></html>",
        title, title
    )
    .unwrap();
}

/// Creates a docs html dir with a versions.json manifest and two versions.
fn build_versioned_docs_dir() -> tempfile::TempDir {
    let html_dir = tempfile::tempdir().unwrap();
    fs::write(
        html_dir.path().join("versions.json"),
        r#"{"versions": ["0.2.0", "0.1.0"], "latest": "0.2.0"}"#,
    )
    .unwrap();

    let v2_en = html_dir.path().join("0.2.0").join("en");
    let v2_zh = html_dir.path().join("0.2.0").join("zh");
    let v1_en = html_dir.path().join("0.1.0").join("en");
    fs::create_dir_all(&v2_en).unwrap();
    fs::create_dir_all(&v2_zh).unwrap();
    fs::create_dir_all(&v1_en).unwrap();

    write_test_doc(&v2_en, "quick_start", "Quick Start v2");
    write_test_doc(&v2_zh, "quick_start", "Quick Start v2 zh");
    write_test_doc(&v1_en, "quick_start", "Quick Start v1");

    html_dir
}

async fn response_json(response: axum::response::Response) -> serde_json::Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&body).unwrap()
}

#[tokio::test]
async fn list_docs_returns_empty_when_no_docs() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert!(json["zh"].as_array().unwrap().is_empty());
    assert!(json["en"].as_array().unwrap().is_empty());
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn get_doc_returns_html() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = tempfile::tempdir().unwrap();
    let zh_dir = html_dir.path().join("zh");
    fs::create_dir_all(&zh_dir).unwrap();
    write_test_doc(&zh_dir, "quick_start", "Quick Start");

    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

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
    let json = response_json(response).await;
    assert!(json["html"].as_str().unwrap().contains("Quick Start"));
    // Legacy layout is served as the implicit "latest" version.
    assert_eq!(json["version"].as_str().unwrap(), "latest");
}

#[tokio::test]
async fn list_docs_uses_default_version() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["version"].as_str().unwrap(), "0.2.0");
    let en = json["en"].as_array().unwrap();
    assert_eq!(en.len(), 1);
    assert_eq!(en[0]["title"].as_str().unwrap(), "Quick Start v2");
    let zh = json["zh"].as_array().unwrap();
    assert_eq!(zh.len(), 1);
}

#[tokio::test]
async fn list_docs_with_version_query() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs?version=0.1.0")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["version"].as_str().unwrap(), "0.1.0");
    let en = json["en"].as_array().unwrap();
    assert_eq!(en[0]["title"].as_str().unwrap(), "Quick Start v1");
}

#[tokio::test]
async fn list_docs_with_unknown_version_returns_404() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs?version=9.9.9")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_versions_returns_all_versions() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs/versions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(
        json["versions"].as_array().unwrap(),
        &vec![serde_json::json!("0.2.0"), serde_json::json!("0.1.0")]
    );
    assert_eq!(json["latest"].as_str().unwrap(), "0.2.0");
}

#[tokio::test]
async fn get_doc_uses_default_version() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs/en/quick_start")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["version"].as_str().unwrap(), "0.2.0");
    assert_eq!(json["title"].as_str().unwrap(), "Quick Start v2");
}

#[tokio::test]
async fn get_versioned_doc_returns_requested_version() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs/0.1.0/en/quick_start")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = response_json(response).await;
    assert_eq!(json["version"].as_str().unwrap(), "0.1.0");
    assert_eq!(json["title"].as_str().unwrap(), "Quick Start v1");
}

#[tokio::test]
async fn get_versioned_doc_unknown_version_returns_404() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs/9.9.9/en/quick_start")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_versioned_doc_unknown_doc_returns_404() {
    let (db, _tmp) = common::setup_test_db().await;
    let html_dir = build_versioned_docs_dir();
    let app =
        common::build_test_app_with_docs(db, html_dir.path().to_str().unwrap().to_string()).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/docs/0.2.0/en/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
