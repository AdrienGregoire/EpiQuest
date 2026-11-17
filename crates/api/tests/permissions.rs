/*
 ** EPITECH PROJECT, 2026
 ** permissions.rs
 ** File description:
 ** integration tests for the role middlewares
 */

mod common;

use api::create_app;
use axum::{Router, http::StatusCode};
use common::{call, json_body, register_and_login, setup_test_pool};
use serde_json::json;

#[derive(Clone, Copy)]
enum Min {
    Member,
    Gm,
}

const CAMPAIGN_ROUTES: &[(&str, &str, Min)] = &[
    ("GET", "/campaigns/{id}", Min::Member),
    ("GET", "/campaigns/{id}/members", Min::Member),
    ("PATCH", "/campaigns/{id}", Min::Gm),
    ("POST", "/campaigns/{id}/invites", Min::Gm),
    ("GET", "/campaigns/{id}/invites", Min::Gm),
    ("DELETE", "/campaigns/{id}/invites/{invite_id}", Min::Gm),
    ("DELETE", "/campaigns/{id}", Min::Gm),
];

const AUTHENTICATED_ROUTES: &[(&str, &str)] = &[
    ("GET", "/auth/me"),
    ("POST", "/auth/logout"),
    ("POST", "/auth/logout-all"),
    ("GET", "/campaigns"),
    ("POST", "/campaigns"),
    ("GET", "/campaigns/joined"),
    ("POST", "/campaigns/join"),
];

struct Setup {
    gm: String,
    player: String,
    stranger: String,
    campaign_id: String,
}

fn fill(template: &str, campaign_id: &str) -> String {
    template
        .replace("{id}", campaign_id)
        .replace("{invite_id}", &uuid::Uuid::new_v4().to_string())
}

async fn status(app: &Router, method: &str, uri: &str, token: Option<&str>) -> StatusCode {
    call(app, method, uri, Some(json!({})), token)
        .await
        .status()
}

fn is_allowed(status: StatusCode) -> bool {
    status != StatusCode::UNAUTHORIZED
        && status != StatusCode::FORBIDDEN
        && !status.is_server_error()
}

async fn create_campaign(app: &Router, token: &str) -> String {
    let payload = json!({ "name": "Matrice" });
    let response = call(app, "POST", "/campaigns", Some(payload), Some(token)).await;

    assert_eq!(response.status(), StatusCode::CREATED);
    json_body(response).await["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn join(app: &Router, gm: &str, player: &str, campaign_id: &str) {
    let uri = format!("/campaigns/{campaign_id}/invites");
    let invite = call(app, "POST", &uri, None, Some(gm)).await;
    let code = json_body(invite).await["code"]
        .as_str()
        .unwrap()
        .to_string();
    let payload = json!({ "code": code });
    let response = call(app, "POST", "/campaigns/join", Some(payload), Some(player)).await;

    assert_eq!(response.status(), StatusCode::CREATED);
}

async fn setup(app: &Router) -> Setup {
    let gm = register_and_login(app).await;
    let player = register_and_login(app).await;
    let stranger = register_and_login(app).await;
    let campaign_id = create_campaign(app, &gm).await;

    join(app, &gm, &player, &campaign_id).await;
    Setup {
        gm,
        player,
        stranger,
        campaign_id,
    }
}

#[tokio::test]
async fn test_campaign_routes_enforce_roles() {
    let app = create_app(setup_test_pool().await);
    let s = setup(&app).await;

    for &(method, template, min) in CAMPAIGN_ROUTES {
        let uri = fill(template, &s.campaign_id);
        let label = format!("{method} {template}");
        let anonymous = status(&app, method, &uri, None).await;

        assert_eq!(anonymous, StatusCode::UNAUTHORIZED, "{label} as anonymous");
        let stranger = status(&app, method, &uri, Some(&s.stranger)).await;
        assert_eq!(stranger, StatusCode::FORBIDDEN, "{label} as stranger");
        let player = status(&app, method, &uri, Some(&s.player)).await;
        match min {
            Min::Gm => assert_eq!(player, StatusCode::FORBIDDEN, "{label} as player"),
            Min::Member => assert!(is_allowed(player), "{label} as player: {player}"),
        }
        let gm = status(&app, method, &uri, Some(&s.gm)).await;
        assert!(is_allowed(gm), "{label} as gm: {gm}");
    }
}

#[tokio::test]
async fn test_authenticated_routes_reject_anonymous() {
    let app = create_app(setup_test_pool().await);

    for &(method, path) in AUTHENTICATED_ROUTES {
        let anonymous = status(&app, method, path, None).await;
        assert_eq!(anonymous, StatusCode::UNAUTHORIZED, "{method} {path}");

        let forged = status(&app, method, path, Some("not_a_real_token")).await;
        assert_eq!(forged, StatusCode::UNAUTHORIZED, "{method} {path} forged");
    }
}

#[tokio::test]
async fn test_unknown_or_malformed_campaign_id_is_not_found() {
    let app = create_app(setup_test_pool().await);
    let token = register_and_login(&app).await;

    for &(method, template, _) in CAMPAIGN_ROUTES {
        let unknown = fill(template, &uuid::Uuid::new_v4().to_string());
        let response = status(&app, method, &unknown, Some(&token)).await;
        assert_eq!(
            response,
            StatusCode::NOT_FOUND,
            "{method} {template} unknown"
        );

        let malformed = fill(template, "not-a-uuid");
        let response = status(&app, method, &malformed, Some(&token)).await;
        assert_eq!(
            response,
            StatusCode::NOT_FOUND,
            "{method} {template} malformed"
        );
    }
}

#[tokio::test]
async fn test_role_depends_on_the_campaign() {
    let app = create_app(setup_test_pool().await);
    let alice = register_and_login(&app).await;
    let bob = register_and_login(&app).await;
    let alices = create_campaign(&app, &alice).await;
    let bobs = create_campaign(&app, &bob).await;

    join(&app, &bob, &alice, &bobs).await;
    let patch = json!({ "name": "Renommée" });
    let uri = format!("/campaigns/{alices}");
    let own = call(&app, "PATCH", &uri, Some(patch.clone()), Some(&alice)).await;
    assert_eq!(own.status(), StatusCode::OK);
    let uri = format!("/campaigns/{bobs}");
    let read = call(&app, "GET", &uri, None, Some(&alice)).await;
    assert_eq!(read.status(), StatusCode::OK);
    let write = call(&app, "PATCH", &uri, Some(patch), Some(&alice)).await;
    assert_eq!(write.status(), StatusCode::FORBIDDEN);
}
