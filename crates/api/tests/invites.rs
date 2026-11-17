/*
 ** EPITECH PROJECT, 2026
 ** invites.rs
 ** File description:
 ** integration tests for invites and joining a campaign
 */

mod common;

use api::create_app;
use axum::{Router, http::StatusCode, response::Response};
use common::{call, json_body, register_and_login, setup_test_pool};
use serde_json::{Value, json};

async fn create_campaign(app: &Router, token: &str) -> Value {
    let payload = json!({ "name": "La Tour sombre" });
    let response = call(app, "POST", "/campaigns", Some(payload), Some(token)).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await
}

async fn create_invite(app: &Router, token: &str, campaign: &Value) -> Value {
    let uri = format!("/campaigns/{}/invites", campaign["id"].as_str().unwrap());
    let response = call(app, "POST", &uri, None, Some(token)).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await
}

async fn join(app: &Router, token: &str, code: &str) -> Response {
    let payload = json!({ "code": code });

    call(app, "POST", "/campaigns/join", Some(payload), Some(token)).await
}

#[tokio::test]
async fn test_only_gm_manages_invites() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let other = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm).await;
    let uri = format!("/campaigns/{}/invites", campaign["id"].as_str().unwrap());
    let anonymous = call(&app, "POST", &uri, None, None).await;

    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    let forbidden = call(&app, "POST", &uri, None, Some(&other)).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let forbidden = call(&app, "GET", &uri, None, Some(&other)).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let invite = create_invite(&app, &gm, &campaign).await;
    let code = invite["code"].as_str().unwrap();
    assert_eq!(code.len(), 8);
    assert!(
        code.chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
    );
    assert_eq!(invite["campaign_id"], campaign["id"]);
    let listed = json_body(call(&app, "GET", &uri, None, Some(&gm)).await).await;
    assert_eq!(listed.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_join_with_code_adds_player() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let player = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm).await;
    let id = campaign["id"].as_str().unwrap();
    let invite = create_invite(&app, &gm, &campaign).await;
    let code = invite["code"].as_str().unwrap();
    let response = join(&app, &player, &format!("  {}  ", code.to_lowercase())).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(json_body(response).await["id"], campaign["id"]);
    let joined = call(&app, "GET", "/campaigns/joined", None, Some(&player)).await;
    let joined = json_body(joined).await;
    assert_eq!(joined.as_array().unwrap().len(), 1);
    assert_eq!(joined[0]["id"], campaign["id"]);
    let uri = format!("/campaigns/{id}");
    let read = call(&app, "GET", &uri, None, Some(&player)).await;
    assert_eq!(read.status(), StatusCode::OK);
    let patch = json!({ "name": "Hack" });
    let patch = call(&app, "PATCH", &uri, Some(patch), Some(&player)).await;
    assert_eq!(patch.status(), StatusCode::FORBIDDEN);
    let delete = call(&app, "DELETE", &uri, None, Some(&player)).await;
    assert_eq!(delete.status(), StatusCode::FORBIDDEN);
    let mine = call(&app, "GET", "/campaigns", None, Some(&player)).await;
    assert!(json_body(mine).await.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_same_user_is_gm_here_and_player_elsewhere() {
    let app = create_app(setup_test_pool().await);
    let alice = register_and_login(&app).await;
    let bob = register_and_login(&app).await;
    let alices = create_campaign(&app, &alice).await;
    let bobs = create_campaign(&app, &bob).await;
    let invite = create_invite(&app, &bob, &bobs).await;
    let response = join(&app, &alice, invite["code"].as_str().unwrap()).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let as_gm = call(&app, "GET", "/campaigns", None, Some(&alice)).await;
    let as_gm = json_body(as_gm).await;
    assert_eq!(as_gm.as_array().unwrap().len(), 1);
    assert_eq!(as_gm[0]["id"], alices["id"]);
    let as_player = call(&app, "GET", "/campaigns/joined", None, Some(&alice)).await;
    let as_player = json_body(as_player).await;
    assert_eq!(as_player.as_array().unwrap().len(), 1);
    assert_eq!(as_player[0]["id"], bobs["id"]);
}

#[tokio::test]
async fn test_join_rejects_bad_requests() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let player = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm).await;
    let invite = create_invite(&app, &gm, &campaign).await;
    let code = invite["code"].as_str().unwrap();
    let payload = json!({ "code": code });
    let anonymous = call(&app, "POST", "/campaigns/join", Some(payload), None).await;

    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    let unknown = join(&app, &player, "ZZZZZZZZ").await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    let empty = join(&app, &player, "").await;
    assert_eq!(empty.status(), StatusCode::NOT_FOUND);
    let own = join(&app, &gm, code).await;
    assert_eq!(own.status(), StatusCode::CONFLICT);
    let first = join(&app, &player, code).await;
    assert_eq!(first.status(), StatusCode::CREATED);
    let twice = join(&app, &player, code).await;
    assert_eq!(twice.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_revoked_invite_cannot_be_used() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let other = register_and_login(&app).await;
    let player = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm).await;
    let id = campaign["id"].as_str().unwrap();
    let invite = create_invite(&app, &gm, &campaign).await;
    let code = invite["code"].as_str().unwrap();
    let uri = format!("/campaigns/{id}/invites/{}", invite["id"].as_str().unwrap());
    let forbidden = call(&app, "DELETE", &uri, None, Some(&other)).await;

    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let unknown_uri = format!("/campaigns/{id}/invites/{}", uuid::Uuid::new_v4());
    let unknown = call(&app, "DELETE", &unknown_uri, None, Some(&gm)).await;
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    let revoked = call(&app, "DELETE", &uri, None, Some(&gm)).await;
    assert_eq!(revoked.status(), StatusCode::NO_CONTENT);
    let again = call(&app, "DELETE", &uri, None, Some(&gm)).await;
    assert_eq!(again.status(), StatusCode::NOT_FOUND);
    let response = join(&app, &player, code).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let list_uri = format!("/campaigns/{id}/invites");
    let listed = json_body(call(&app, "GET", &list_uri, None, Some(&gm)).await).await;
    assert!(listed.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_expired_invite_cannot_be_used() {
    let pool = setup_test_pool().await;
    let app = create_app(pool.clone());
    let gm = register_and_login(&app).await;
    let player = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm).await;
    let invite = create_invite(&app, &gm, &campaign).await;
    let code = invite["code"].as_str().unwrap();

    sqlx::query(
        "UPDATE campaign_invites SET expires_at = NOW() - INTERVAL '1 second' WHERE code = $1",
    )
    .bind(code)
    .execute(&pool)
    .await
    .unwrap();
    let response = join(&app, &player, code).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_members_are_visible_to_members_only() {
    let app = create_app(setup_test_pool().await);
    let gm = register_and_login(&app).await;
    let player = register_and_login(&app).await;
    let stranger = register_and_login(&app).await;
    let campaign = create_campaign(&app, &gm).await;
    let invite = create_invite(&app, &gm, &campaign).await;
    let uri = format!("/campaigns/{}/members", campaign["id"].as_str().unwrap());
    let before = call(&app, "GET", &uri, None, Some(&player)).await;

    assert_eq!(before.status(), StatusCode::FORBIDDEN);
    let response = join(&app, &player, invite["code"].as_str().unwrap()).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let forbidden = call(&app, "GET", &uri, None, Some(&stranger)).await;
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    for token in [&gm, &player] {
        let response = call(&app, "GET", &uri, None, Some(token)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let members = json_body(response).await;
        assert_eq!(members.as_array().unwrap().len(), 2);
        assert_eq!(members[0]["role"], "gm");
        assert_eq!(members[1]["role"], "player");
    }
}
