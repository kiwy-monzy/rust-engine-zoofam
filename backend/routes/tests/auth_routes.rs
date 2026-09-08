use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use auth::Jwt;
use routes::AppState;

fn state() -> AppState {
    let pool = db::create_pool_from(":memory:").expect("pool");
    db::run_migrations(&pool).expect("migrations");
    AppState::new(pool, Jwt::new("test-secret", 1, 7))
}

async fn call(state: &AppState, request: Request<Body>) -> (StatusCode, Value) {
    let response = routes::app(state.clone())
        .oneshot(request)
        .await
        .expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

fn post(path: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn post_raw(path: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn get(path: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri(path);
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder.body(Body::empty()).unwrap()
}

fn post_auth(path: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

async fn register_and_login(state: &AppState, email: &str) -> String {
    let (status, _) = call(
        state,
        post(
            "/api/v1/auth/register",
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = call(
        state,
        post(
            "/api/v1/auth/login",
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body["token"].as_str().expect("token").to_string()
}

#[tokio::test]
async fn health_needs_no_token() {
    let state = state();
    let (status, body) = call(&state, get("/api/v1/health", None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn a_private_route_without_a_token_is_unauthorised() {
    let state = state();
    let (status, _) = call(&state, get("/api/v1/users", None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn a_private_route_with_a_forged_token_is_unauthorised() {
    let state = state();
    let forged = Jwt::new("not-the-secret", 1, 7)
        .issue(
            "00000000-0000-0000-0000-000000000000",
            "x@y.z",
            vec![],
            vec![],
            0,
        )
        .unwrap();
    let (status, _) = call(&state, get("/api/v1/users", Some(&forged))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_expired_token_is_unauthorised() {
    let state = state();
    let stale = Jwt::new("test-secret", -1, 7)
        .issue(
            "00000000-0000-0000-0000-000000000000",
            "x@y.z",
            vec![],
            vec![],
            0,
        )
        .unwrap();
    let (status, body) = call(&state, get("/api/v1/users", Some(&stale))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body["error"].as_str().unwrap().contains("expired"));
}

#[tokio::test]
async fn registering_then_logging_in_returns_a_token_that_opens_me() {
    let state = state();
    let token = register_and_login(&state, "first@example.com").await;

    let (status, body) = call(&state, get("/api/v1/auth/me", Some(&token))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["email"], "first@example.com");
}

#[tokio::test]
async fn a_signed_in_user_with_no_role_is_forbidden_not_unauthorised() {
    let state = state();
    let token = register_and_login(&state, "nobody@example.com").await;

    let (status, body) = call(&state, get("/api/v1/users", Some(&token))).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "authenticated, but not permitted"
    );
    assert!(body["error"].as_str().unwrap().contains("users:read"));
}

#[tokio::test]
async fn the_admin_role_carries_the_permissions_the_migrations_granted() {
    let state = state();
    let token = register_and_login(&state, "admin@example.com").await;

    let user = controller::users::find_by_email(&state.pool, "admin@example.com").unwrap();
    let admin = controller::roles::list(&state.pool)
        .unwrap()
        .into_iter()
        .find(|r| r.role.name == "admin")
        .expect("the admin role is seeded by migration 2");
    controller::users::assign_role(&state.pool, &user.id, admin.role.id).unwrap();

    let (_, body) = call(
        &state,
        post(
            "/api/v1/auth/login",
            json!({ "email": "admin@example.com", "password": "password123" }),
        ),
    )
    .await;
    let fresh = body["token"].as_str().unwrap();

    let (status, body) = call(&state, get("/api/v1/users", Some(fresh))).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["users"].as_array().unwrap().len() >= 1);

    let (status, _) = call(&state, get("/api/v1/users", Some(&token))).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "the token issued before the role was granted must not gain it"
    );
}

#[tokio::test]
async fn the_viewer_role_may_read_but_not_write() {
    let state = state();
    register_and_login(&state, "viewer@example.com").await;

    let user = controller::users::find_by_email(&state.pool, "viewer@example.com").unwrap();
    let viewer = controller::roles::list(&state.pool)
        .unwrap()
        .into_iter()
        .find(|r| r.role.name == "viewer")
        .expect("the viewer role is seeded by migration 2");
    controller::users::assign_role(&state.pool, &user.id, viewer.role.id).unwrap();

    let (_, body) = call(
        &state,
        post(
            "/api/v1/auth/login",
            json!({ "email": "viewer@example.com", "password": "password123" }),
        ),
    )
    .await;
    let token = body["token"].as_str().unwrap().to_string();

    let (status, _) = call(&state, get("/api/v1/users", Some(&token))).await;
    assert_eq!(status, StatusCode::OK);

    let request = Request::builder()
        .method("POST")
        .uri("/api/v1/users")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(
            json!({ "email": "new@example.com", "password": "password123" }).to_string(),
        ))
        .unwrap();
    let (status, _) = call(&state, request).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn the_wrong_password_is_rejected_and_says_nothing_useful() {
    let state = state();
    register_and_login(&state, "someone@example.com").await;

    let (status, body) = call(
        &state,
        post(
            "/api/v1/auth/login",
            json!({ "email": "someone@example.com", "password": "wrong" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "the email or password is wrong");

    let (status, body) = call(
        &state,
        post(
            "/api/v1/auth/login",
            json!({ "email": "nobody@example.com", "password": "wrong" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        body["error"], "the email or password is wrong",
        "an unknown email must not be distinguishable from a wrong password"
    );
}

#[tokio::test]
async fn the_same_email_cannot_register_twice() {
    let state = state();
    register_and_login(&state, "dupe@example.com").await;

    let (status, _) = call(
        &state,
        post(
            "/api/v1/auth/register",
            json!({ "email": "DUPE@example.com", "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "email match is case-insensitive"
    );
}

#[tokio::test]
async fn a_short_password_is_refused() {
    let state = state();
    let (status, _) = call(
        &state,
        post(
            "/api/v1/auth/register",
            json!({ "email": "short@example.com", "password": "123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn an_oversized_password_is_refused() {
    let state = state();
    let (status, body) = call(
        &state,
        post(
            "/api/v1/auth/register",
            json!({ "email": "huge@example.com", "password": "x".repeat(129) }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "the password must be at most 128 characters");
}

#[tokio::test]
async fn html_in_the_display_name_is_refused_not_stored() {
    let state = state();
    let (status, body) = call(
        &state,
        post(
            "/api/v1/auth/register",
            json!({
                "email": "xss@example.com",
                "password": "password123",
                "display_name": "<script>alert(1)</script>",
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "HTML/XSS content is not allowed");
}

#[tokio::test]
async fn an_oversized_display_name_is_refused() {
    let state = state();
    let (status, _) = call(
        &state,
        post(
            "/api/v1/auth/register",
            json!({
                "email": "longname@example.com",
                "password": "password123",
                "display_name": "a".repeat(121),
            }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_malformed_body_is_rejected_without_parser_detail() {
    let state = state();
    let (status, body) = call(
        &state,
        post_raw("/api/v1/auth/login", "{\"email\": \"a@b.c\","),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let text = body.to_string();
    assert_eq!(body["error"], "the request body is not valid JSON");
    assert!(!text.contains("EOF"), "serde detail leaked: {text}");
    assert!(!text.contains("parsing"), "serde detail leaked: {text}");
}

#[tokio::test]
async fn a_wrong_typed_field_is_rejected_without_serde_detail() {
    let state = state();
    let (status, body) = call(
        &state,
        post(
            "/api/v1/auth/login",
            json!({ "email": "a@b.c", "password": 12345678 }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let text = body.to_string();
    assert_eq!(
        body["error"],
        "the request body fields have the wrong types or are missing"
    );
    assert!(
        !text.contains("invalid type"),
        "serde detail leaked: {text}"
    );
    assert!(!text.contains("integer"), "serde detail leaked: {text}");
}

fn b64url_json(value: &Value) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let bytes = value.to_string().into_bytes();
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b1 = chunk[0] as u32;
        let b2 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b3 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b1 << 16) | (b2 << 8) | b3;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[n as usize & 63] as char);
        }
    }
    out
}

#[tokio::test]
async fn an_alg_none_token_is_rejected_with_a_generic_message() {
    let state = state();
    // exp/iat far in the future/past do not matter: the unsigned header must
    // fail before any claim is read.
    let header = b64url_json(&json!({ "alg": "none", "typ": "JWT" }));
    let claims = b64url_json(&json!({
        "sub": "00000000-0000-0000-0000-000000000000",
        "email": "attacker@example.com",
        "roles": ["admin"],
        "perms": ["users:read", "users:write"],
        "jti": "forged-jti",
        "ver": 0,
        "exp": i64::MAX / 2,
        "iat": 0,
    }));
    let forged = format!("{header}.{claims}.");

    let (status, body) = call(&state, get("/api/v1/users", Some(&forged))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let text = body.to_string();
    assert_eq!(body["error"], "the token is not valid");
    assert!(
        !text.contains("unknown variant"),
        "library detail leaked: {text}"
    );
    assert!(!text.contains("HS256"), "library detail leaked: {text}");
}

#[tokio::test]
async fn a_duplicate_registration_conflict_names_nothing_internal() {
    let state = state();
    register_and_login(&state, "conflict@example.com").await;

    let (status, body) = call(
        &state,
        post(
            "/api/v1/auth/register",
            json!({ "email": "conflict@example.com", "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"], "that email is already registered");
    let text = body.to_string();
    assert!(!text.contains("gateway_users"), "table name leaked: {text}");
    assert!(!text.contains("_key"), "index name leaked: {text}");
}

#[tokio::test]
async fn the_password_hash_never_leaves_the_server() {
    let state = state();
    let token = register_and_login(&state, "secret@example.com").await;
    let (_, body) = call(&state, get("/api/v1/auth/me", Some(&token))).await;
    let text = body.to_string();
    assert!(!text.contains("password"), "leaked: {text}");
    assert!(!text.contains("argon2"), "leaked: {text}");
}

#[tokio::test]
async fn logout_revokes_the_token_it_was_called_with() {
    let state = state();
    let token = register_and_login(&state, "bye@example.com").await;

    let (status, _) = call(&state, get("/api/v1/auth/me", Some(&token))).await;
    assert_eq!(status, StatusCode::OK, "token works before logout");

    let (status, _) = call(&state, post_auth("/api/v1/auth/logout", &token)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = call(&state, get("/api/v1/auth/me", Some(&token))).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "the same token no longer works"
    );
    assert!(body["error"].as_str().unwrap().contains("signed out"));
}

#[tokio::test]
async fn logout_revokes_only_that_session_not_the_others() {
    let state = state();
    register_and_login(&state, "multi@example.com").await;

    let login = || async {
        let (_, body) = call(
            &state,
            post(
                "/api/v1/auth/login",
                json!({ "email": "multi@example.com", "password": "password123" }),
            ),
        )
        .await;
        body["token"].as_str().unwrap().to_string()
    };
    let phone = login().await;
    let laptop = login().await;

    let (status, _) = call(&state, post_auth("/api/v1/auth/logout", &phone)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = call(&state, get("/api/v1/auth/me", Some(&phone))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "the phone session is out");

    let (status, _) = call(&state, get("/api/v1/auth/me", Some(&laptop))).await;
    assert_eq!(status, StatusCode::OK, "the laptop session is untouched");
}

#[tokio::test]
async fn logout_all_revokes_every_existing_session() {
    let state = state();
    register_and_login(&state, "everywhere@example.com").await;

    let login = || async {
        let (_, body) = call(
            &state,
            post(
                "/api/v1/auth/login",
                json!({ "email": "everywhere@example.com", "password": "password123" }),
            ),
        )
        .await;
        body["token"].as_str().unwrap().to_string()
    };
    let first = login().await;
    let second = login().await;

    let (status, _) = call(&state, post_auth("/api/v1/auth/logout-all", &first)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    for (label, token) in [("first", &first), ("second", &second)] {
        let (status, _) = call(&state, get("/api/v1/auth/me", Some(token))).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{label} session must be revoked"
        );
    }

    let fresh = login().await;
    let (status, _) = call(&state, get("/api/v1/auth/me", Some(&fresh))).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "a login after logout-all works again"
    );
}

#[tokio::test]
async fn a_disabled_account_is_rejected_on_the_next_request() {
    let state = state();
    let token = register_and_login(&state, "disable-me@example.com").await;

    let user = controller::users::find_by_email(&state.pool, "disable-me@example.com").unwrap();
    controller::users::update(
        &state.pool,
        &user.id,
        models::UpdateUser {
            display_name: None,
            avatar_url: None,
            is_active: Some(false),
            first_name: None,
            middle_name: None,
            last_name: None,
            username: None,
        },
    )
    .unwrap();

    let (status, body) = call(&state, get("/api/v1/auth/me", Some(&token))).await;
    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "disabling ends the session immediately"
    );
    assert!(body["error"].as_str().unwrap().contains("disabled"));
}

/// Registers `email`, grants the seeded admin role in the database, and logs
/// in again so the fresh token carries the frozen permission list.
async fn admin_token(state: &AppState, email: &str) -> String {
    call(
        state,
        post(
            "/api/v1/auth/register",
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    let user = controller::users::find_by_email(&state.pool, email).unwrap();
    let admin = controller::roles::list(&state.pool)
        .unwrap()
        .into_iter()
        .find(|r| r.role.name == "admin")
        .expect("the admin role is seeded by migration 2");
    controller::users::assign_role(&state.pool, &user.id, admin.role.id).unwrap();

    let (_, body) = call(
        state,
        post(
            "/api/v1/auth/login",
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    body["token"].as_str().expect("token").to_string()
}

fn post_auth_json(path: &str, token: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[tokio::test]
async fn assigning_an_unknown_role_is_a_404_not_a_500() {
    let state = state();
    let token = admin_token(&state, "assigner@example.com").await;
    let user = controller::users::find_by_email(&state.pool, "assigner@example.com").unwrap();

    let (status, body) = call(
        &state,
        post_auth_json(
            &format!("/api/v1/users/{}/roles/9999", user.id),
            &token,
            json!(null),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "no role with that id");
}

#[tokio::test]
async fn granting_an_unknown_permission_is_a_404_not_a_500() {
    let state = state();
    let token = admin_token(&state, "granter@example.com").await;
    controller::roles::create(
        &state.pool,
        models::CreateRole {
            name: "probe".into(),
            description: String::new(),
        },
    )
    .unwrap();
    let probe = controller::roles::list(&state.pool)
        .unwrap()
        .into_iter()
        .find(|r| r.role.name == "probe")
        .unwrap();

    let (status, body) = call(
        &state,
        post_auth_json(
            &format!("/api/v1/roles/{}/permissions/9999", probe.role.id),
            &token,
            json!(null),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "no permission with that id");
}

#[tokio::test]
async fn permission_writes_need_permissions_write_not_roles_write() {
    let state = state();
    // A real account whose token carries roles:write but NOT permissions:write.
    register_and_login(&state, "legacy-admin@example.com").await;
    let user = controller::users::find_by_email(&state.pool, "legacy-admin@example.com").unwrap();
    let token = state
        .jwt
        .issue(
            &user.id,
            "legacy-admin@example.com",
            vec![],
            vec!["roles:write".into()],
            0,
        )
        .unwrap();

    let (status, body) = call(
        &state,
        post_auth_json(
            "/api/v1/permissions",
            &token,
            json!({ "module": "x", "action": "y", "description": "" }),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "roles:write must not authorize creating permissions"
    );
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("permissions:write"));
}
