/*
 ** EPITECH PROJECT, 2026
 ** campaigns.rs
 ** File description:
 ** integration tests for campaign CRUD
 */

mod common;

use api::create_app;
use axum::{Router, http::StatusCode};
use common::{call, json_body, register_and_login, setup_test_pool};
use serde_json::{Value, json};

async fn create_campaign(app: &Router, token: &str, name: &str) -> Value {
    let payload = json!({ "name": name, "description": "A dark tale" });
    let response = call(app, "POST", "/campaigns", Some(payload), Some(token)).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await
}

fn campaign_uri(campaign: &Value) -> String {
    format!("/campaigns/{}", campaign["id"].as_str().unwrap())
}

#[tokio::test]
async fn test_campaign_routes_require_auth() {
    let app = create_app(setup_test_pool().await);
    let payload = json!({ "name": "Nope" });
    let create = call(&app, "POST", "/campaigns", Some(payload), None).await;

    assert_eq!(create.status(), StatusCode::UNAUTHORIZED);
    let list = call(&app, "GET", "/campaigns", None, None).await;
    assert_eq!(list.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_campaign_sets_creator_as_gm() {
    let app = create_app(setup_test_pool().await);
    let token = register_and_login(&app).await;
    let me = json_body(call(&app, "GET", "/auth/me", None, Some(&token)).await).await;
    let campaign = create_campaign(&app, &token, "  La Tour sombre  ").await;

    assert_eq!(campaign["gm_id"], me["id"]);
    assert_eq!(campaign["name"], "La Tour sombre");
    assert_eq!(campaign["description"], "A dark tale");
}

#[tokio::test]
async fn test_create_campaign_validation() {
    let app = create_app(setup_test_pool().await);
    let token = register_and_login(&app).await;
    let empty = json!({ "name": "   " });
    let response = call(&app, "POST", "/campaigns", Some(empty), Some(&token)).await;

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let too_long = json!({ "name": "a".repeat(101) });
    let response = call(&app, "POST", "/campaigns", Some(too_long), Some(&token)).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let no_description = json!({ "name": "Sans description" });
    let response = call(
        &app,
        "POST",
        "/campaigns",
        Some(no_description),
        Some(&token),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_list_returns_only_my_campaigns() {
    let app = create_app(setup_test_pool().await);
    let alice = register_and_login(&app).await;
    let bob = register_and_login(&app).await;

    create_campaign(&app, &alice, "Alice 1").await;
    create_campaign(&app, &alice, "Alice 2").await;
    create_campaign(&app, &bob, "Bob 1").await;
    let response = call(&app, "GET", "/campaigns", None, Some(&alice)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let list = json_body(response).await;
    assert_eq!(list.as_array().unwrap().len(), 2);
    let response = call(&app, "GET", "/campaigns", None, Some(&bob)).await;
    assert_eq!(json_body(response).await.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_get_campaign_access_rules() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let other = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm, "Secrète").await;
    let uri = campaign_uri(&campaign);
    let owner = call(&app, "GET", &uri, None, Some(&gm)).await;

    assert_eq!(owner.status(), StatusCode::OK);
    let stranger = call(&app, "GET", &uri, None, Some(&other)).await;
    assert_eq!(stranger.status(), StatusCode::FORBIDDEN);
    let unknown_uri = format!("/campaigns/{}", uuid::Uuid::new_v4());
    let missing = call(&app, "GET", &unknown_uri, None, Some(&gm)).await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_campaign() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let other = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm, "Avant").await;
    let uri = campaign_uri(&campaign);
    let patch = json!({ "name": "Après" });
    let response = call(&app, "PATCH", &uri, Some(patch.clone()), Some(&gm)).await;

    assert_eq!(response.status(), StatusCode::OK);
    let updated = json_body(response).await;
    assert_eq!(updated["name"], "Après");
    assert_eq!(updated["description"], "A dark tale");
    let forbidden = call(&app, "PATCH", &uri, Some(patch), Some(&other)).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let nothing = call(&app, "PATCH", &uri, Some(json!({})), Some(&gm)).await;
    assert_eq!(nothing.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let blank = json!({ "name": "  " });
    let invalid = call(&app, "PATCH", &uri, Some(blank), Some(&gm)).await;
    assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_delete_campaign() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let other = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm, "À supprimer").await;
    let uri = campaign_uri(&campaign);
    let forbidden = call(&app, "DELETE", &uri, None, Some(&other)).await;

    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let still_there = call(&app, "GET", &uri, None, Some(&gm)).await;
    assert_eq!(still_there.status(), StatusCode::OK);
    let deleted = call(&app, "DELETE", &uri, None, Some(&gm)).await;
    assert_eq!(deleted.status(), StatusCode::NO_CONTENT);
    let gone = call(&app, "GET", &uri, None, Some(&gm)).await;
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}
