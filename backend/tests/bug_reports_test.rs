mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use oak_website_backend::entities::bug_reports;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tower::ServiceExt;
use uuid::Uuid;

const BOUNDARY: &str = "testboundary";

fn build_multipart(
    text_fields: &[(&str, &str)],
    file: Option<(&str, &str, &str, &[u8])>,
) -> Vec<u8> {
    let mut body = Vec::new();
    for (name, value) in text_fields {
        body.extend_from_slice(
            format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n{value}\r\n")
                .as_bytes(),
        );
    }
    if let Some((name, filename, content_type, bytes)) = file {
        body.extend_from_slice(
            format!(
                "--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\n"
            )
            .as_bytes(),
        );
        body.extend_from_slice(bytes);
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());
    body
}

fn post_report(body: Vec<u8>) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/v1/bug-reports")
        .header("content-type", format!("multipart/form-data; boundary={BOUNDARY}"))
        .body(Body::from(body))
        .unwrap()
}

#[tokio::test]
async fn submit_requires_title_version_and_content() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let body = build_multipart(&[("title", "crash on export")], None);
    let response = app.oneshot(post_report(body)).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn submit_stores_report_without_attachments() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db.clone()).await;

    let body = build_multipart(
        &[
            ("title", "crash on export"),
            ("version", "v0.4.2-alpha"),
            ("content", "steps to reproduce: ..."),
            ("email", "reporter@example.com"),
        ],
        None,
    );
    let response = app.oneshot(post_report(body)).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["id"].is_string());

    let stored = bug_reports::Entity::find().all(&db).await.unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].title, "crash on export");
    assert_eq!(stored[0].app_version, "v0.4.2-alpha");
    assert_eq!(stored[0].email.as_deref(), Some("reporter@example.com"));
    assert!(stored[0].screenshot_key.is_none());
    assert!(stored[0].log_key.is_none());
}

#[tokio::test]
async fn honeypot_submission_fakes_success_and_stores_nothing() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db.clone()).await;

    let body = build_multipart(
        &[
            ("title", "spam"),
            ("version", "v1"),
            ("content", "spam"),
            ("website", "https://spam.example.com"),
        ],
        None,
    );
    let response = app.oneshot(post_report(body)).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let stored = bug_reports::Entity::find().all(&db).await.unwrap();
    assert!(stored.is_empty());
}

#[tokio::test]
async fn screenshot_upload_failure_does_not_lose_report() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db.clone()).await;

    // The test R2 endpoint is unreachable; the report must still be stored,
    // just without the attachment keys.
    let body = build_multipart(
        &[
            ("title", "broken preview"),
            ("version", "v0.4.2-alpha"),
            ("content", "preview pane is black"),
        ],
        Some(("screenshot", "shot.png", "image/png", b"\x89PNG fake")),
    );
    let response = app.oneshot(post_report(body)).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let stored = bug_reports::Entity::find().all(&db).await.unwrap();
    assert_eq!(stored.len(), 1);
    assert!(stored[0].screenshot_key.is_none());
}

#[tokio::test]
async fn non_image_screenshot_is_rejected() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let body = build_multipart(
        &[
            ("title", "x"),
            ("version", "v1"),
            ("content", "y"),
        ],
        Some(("screenshot", "evil.txt", "text/plain", b"nope")),
    );
    let response = app.oneshot(post_report(body)).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_bug_reports_requires_token_and_lists_reports() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = common::build_test_app(db).await;

    let submit = build_multipart(
        &[
            ("title", "admin visible"),
            ("version", "v0.4.2-alpha"),
            ("content", "please fix"),
        ],
        None,
    );
    let response = app.clone().oneshot(post_report(submit)).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let unauthorized = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/bug-reports")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/bug-reports")
                .header("authorization", "Bearer test-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let reports = json["reports"].as_array().unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0]["title"], "admin visible");
    assert_eq!(reports[0]["app_version"], "v0.4.2-alpha");
}

#[tokio::test]
async fn attachment_link_redirects_to_fresh_presigned_url() {
    let (db, _tmp) = common::setup_test_db().await;

    let id = Uuid::new_v4();
    bug_reports::ActiveModel {
        id: Set(id),
        title: Set("with attachment".to_string()),
        app_version: Set("v1".to_string()),
        content: Set("see screenshot".to_string()),
        email: Set(None),
        screenshot_key: Set(Some(format!("bug-reports/{}/screenshot-shot.png", id))),
        screenshot_filename: Set(Some("shot.png".to_string())),
        log_key: Set(None),
        log_filename: Set(None),
        created_at: Set(chrono::Utc::now().into()),
    }
    .insert(&db)
    .await
    .unwrap();

    let app = common::build_test_app(db).await;

    // Presigning is offline crypto, so no real R2 is needed for the redirect.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/bug-reports/{}/files/screenshot", id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = response.headers().get("location").unwrap().to_str().unwrap();
    assert!(location.contains("screenshot-shot.png"));

    // Missing attachment kind -> 404.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/bug-reports/{}/files/log", id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Unknown report id -> 404.
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/bug-reports/{}/files/screenshot", Uuid::new_v4()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
