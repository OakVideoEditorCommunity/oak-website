use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

mod common;
use common::build_test_app;

#[tokio::test]
async fn cors_preflight_returns_204() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = build_test_app(db).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/v1/health")
                .header("origin", "http://localhost:3000")
                .header("access-control-request-method", "GET")
                .header("access-control-request-headers", "content-type")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let (db, _tmp) = common::setup_test_db().await;
    let app = build_test_app(db).await;

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
