//! Wallet route behaviour: RBAC gating, the 503 contract when signing
//! material is absent, and the full signed flow (issue → download → web
//! service → edit → delete) when the certificates are present.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use applewallet::{PassKit, WalletConfig};
use auth::Jwt;
use routes::AppState;

/// Tests run with CWD = crate dir; the repo's signing material lives in
/// `app/priv`. Mirrors production wiring exactly: the kit persists through the
/// same SQLite-backed store the route helpers read, never an in-memory one.
fn state_with_wallet() -> AppState {
    let pool = db::create_pool_from(":memory:").expect("pool");
    db::run_migrations(&pool).expect("migrations");
    let config = WalletConfig {
        priv_dir: std::path::PathBuf::from("../priv"),
        web_service_path: "/api/v1/wallet/v1".to_string(),
        ..WalletConfig::default()
    };
    let kit = PassKit::with_store(
        config,
        Arc::new(controller::wallet::DbStore::new(pool.clone())),
    )
    .ok()
    .map(Arc::new);
    AppState::new(pool, Jwt::new("test-secret", 1, 7)).with_wallet(kit)
}

fn state_without_wallet() -> AppState {
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

async fn call_bytes(state: &AppState, request: Request<Body>) -> (StatusCode, Vec<u8>, String) {
    let response = routes::app(state.clone())
        .oneshot(request)
        .await
        .expect("response");
    let status = response.status();
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    (status, bytes.to_vec(), content_type)
}

fn get(path: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri(path);
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder.body(Body::empty()).unwrap()
}

fn json_request(method: &str, path: &str, token: Option<&str>, body: Value) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

fn raw_request(method: &str, path: &str, token: Option<&str>, body: &str) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn register_and_login(state: &AppState, email: &str) -> String {
    let (status, _) = call(
        state,
        json_request(
            "POST",
            "/api/v1/auth/register",
            None,
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = call(
        state,
        json_request(
            "POST",
            "/api/v1/auth/login",
            None,
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    body["token"].as_str().expect("token").to_string()
}

/// A fresh login for a user holding `role_name`, assigned server-side.
/// Permissions ride inside the JWT, so the role must exist before login.
async fn token_with_role(state: &AppState, email: &str, role_name: &str) -> String {
    let user = controller::users::find_by_email(&state.pool, email).unwrap();
    let role = controller::roles::list(&state.pool)
        .unwrap()
        .into_iter()
        .find(|r| r.role.name == role_name)
        .expect("the role is seeded by migration");
    controller::users::assign_role(&state.pool, &user.id, role.role.id).unwrap();

    let (_, body) = call(
        state,
        json_request(
            "POST",
            "/api/v1/auth/login",
            None,
            json!({ "email": email, "password": "password123" }),
        ),
    )
    .await;
    body["token"].as_str().expect("token").to_string()
}

#[tokio::test]
async fn a_wallet_admin_route_without_a_token_is_unauthorised() {
    let state = state_with_wallet();
    for path in ["/api/v1/wallet/passes", "/api/v1/wallet/samples"] {
        let (status, _) = call(&state, get(path, None)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
    }
}

#[tokio::test]
async fn wallet_reads_need_the_wallet_read_permission() {
    let state = state_with_wallet();
    let token = register_and_login(&state, "nobody@example.com").await;

    let (status, body) = call(&state, get("/api/v1/wallet/samples", Some(&token))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(
        body["error"].as_str().unwrap().contains("wallet:read"),
        "the error must name the missing permission: {body}"
    );
}

#[tokio::test]
async fn a_viewer_may_read_wallet_but_not_issue_passes() {
    let state = state_with_wallet();
    let _ = register_and_login(&state, "viewer@example.com").await;
    let viewer = token_with_role(&state, "viewer@example.com", "viewer").await;

    // Migration 9 grants viewer every wallet read action...
    let (status, _) = call(&state, get("/api/v1/wallet/samples", Some(&viewer))).await;
    assert_eq!(status, StatusCode::OK);

    // ...but issuing is a write and stays admin-only.
    let (status, body) = call(
        &state,
        json_request(
            "POST",
            "/api/v1/wallet/issue",
            Some(&viewer),
            json!({ "sample_id": "coupon" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(
        body["error"].as_str().unwrap().contains("wallet:write"),
        "the error must name the missing permission: {body}"
    );
}

#[tokio::test]
async fn issuing_a_pass_needs_signing_material_and_answers_503_without_it() {
    // RBAC passes, then the missing engine surfaces as 503 — not a 500.
    let state = state_without_wallet();
    let _ = register_and_login(&state, "admin@example.com").await;
    let admin = token_with_role(&state, "admin@example.com", "admin").await;

    let (status, body) = call(
        &state,
        json_request(
            "POST",
            "/api/v1/wallet/issue",
            Some(&admin),
            json!({ "sample_id": "coupon" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert!(body["error"].as_str().unwrap().contains("not configured"));

    let (status, _) = call(&state, get("/api/v1/wallet/passes", Some(&admin))).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

// ------------------------------------------------- signed flow (certs on) ---

/// Issues a coupon as admin and returns (state, admin_token, pass payload).
/// Returns `None` when signing material is absent (test then skips).
async fn issued_coupon() -> Option<(AppState, String, Value)> {
    let state = state_with_wallet();
    if state.wallet.is_none() {
        return None;
    }
    let _ = register_and_login(&state, "admin@example.com").await;
    let admin = token_with_role(&state, "admin@example.com", "admin").await;

    let (status, body) = call(
        &state,
        json_request(
            "POST",
            "/api/v1/wallet/issue",
            Some(&admin),
            json!({ "sample_id": "coupon" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{body}");
    Some((state, admin, body["pass"].clone()))
}

#[tokio::test]
async fn an_issued_pass_downloads_as_a_real_pkpass_archive() {
    let Some((state, _, pass)) = issued_coupon().await else {
        return; // no signing material on this machine
    };

    let url = pass["download_url"].as_str().unwrap();
    let path = url.split_once("/api/v1/wallet/pass/").unwrap().1;
    let (status, bytes, mime) =
        call_bytes(&state, get(&format!("/api/v1/wallet/pass/{path}"), None)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(mime, "application/vnd.apple.pkpass");
    // A zip archive starts with the local-file-header magic "PK\x03\x04".
    assert_eq!(&bytes[..4], b"PK\x03\x04");

    // Unknown serials stay 404, never a 500.
    let (status, _, _) = call_bytes(
        &state,
        get(
            "/api/v1/wallet/pass/coupon/PASS-does-not-exist.pkpass",
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_qr_endpoint_renders_svg_for_an_existing_pass() {
    let Some((state, admin, pass)) = issued_coupon().await else {
        return;
    };
    // URLs carry the stamped pass type identifier, never the sample id.
    let ptype = pass["pass_type_id"].as_str().unwrap();
    let serial = pass["serial_number"].as_str().unwrap();
    let (status, bytes, mime) = call_bytes(
        &state,
        get(
            &format!("/api/v1/wallet/passes/{ptype}/{serial}/qr.svg"),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(mime, "image/svg+xml");
    let svg = String::from_utf8(bytes).unwrap();
    assert!(svg.contains("<svg"));
}

#[tokio::test]
async fn the_update_web_service_authenticates_with_the_apple_pass_token() {
    let Some((state, _, pass)) = issued_coupon().await else {
        return;
    };
    let serial = pass["serial_number"].as_str().unwrap().to_string();
    let token = pass["auth_token"].as_str().unwrap().to_string();
    let ptype = pass["pass_type_id"].as_str().unwrap();
    let registration_path =
        format!("/api/v1/wallet/v1/devices/DEV-42/registrations/{ptype}/{serial}");

    let make_register = |auth_header: Option<String>| {
        let mut b = Request::builder()
            .method("POST")
            .uri(&registration_path)
            .header("content-type", "application/json");
        if let Some(a) = auth_header {
            b = b.header("authorization", a);
        }
        b.body(Body::from(r#"{"pushToken":"a94c0d15f3"}"#)).unwrap()
    };

    // Wrong or missing ApplePass credential → 401.
    let (status, _, _) = call_bytes(&state, make_register(Some("ApplePass nope".into()))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _, _) = call_bytes(&state, make_register(None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // The correct credential registers the device: 201 first, 200 when repeated.
    let good = format!("ApplePass {token}");
    let (status, _, _) = call_bytes(&state, make_register(Some(good.clone()))).await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _, _) = call_bytes(&state, make_register(Some(good))).await;
    assert_eq!(status, StatusCode::OK);

    // Device listing picks up the registered serial.
    let (status, body) = call(
        &state,
        get(
            &format!("/api/v1/wallet/v1/devices/DEV-42/registrations/{ptype}"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["serialNumbers"][0], serial.as_str());

    // Fetching the latest pass needs the ApplePass credential again.
    let latest_path = format!("/api/v1/wallet/v1/passes/{ptype}/{serial}");
    let (status, _, _) = call_bytes(&state, raw_request("GET", &latest_path, None, "")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let latest = Request::builder()
        .method("GET")
        .uri(&latest_path)
        .header("authorization", format!("ApplePass {token}"))
        .body(Body::empty())
        .unwrap();
    let (status, bytes, mime) = call_bytes(&state, latest).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(mime, "application/vnd.apple.pkpass");
    assert_eq!(&bytes[..4], b"PK\x03\x04");

    // Unregistering removes the device from listings.
    let unregister = Request::builder()
        .method("DELETE")
        .uri(&registration_path)
        .header("authorization", format!("ApplePass {token}"))
        .body(Body::empty())
        .unwrap();
    let (status, _) = call(&state, unregister).await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = call(
        &state,
        get(
            &format!("/api/v1/wallet/v1/devices/DEV-42/registrations/{ptype}"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn an_admin_can_edit_then_delete_an_issued_pass() {
    let Some((state, admin, pass)) = issued_coupon().await else {
        return;
    };
    let serial = pass["serial_number"].as_str().unwrap().to_string();
    let ptype = pass["pass_type_id"].as_str().unwrap();

    // Read back the stored JSON...
    let (status, listed) = call(&state, get("/api/v1/wallet/passes", Some(&admin))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed["passes"].as_array().unwrap().len(), 1);
    let (status, body) = call(
        &state,
        get(
            &format!("/api/v1/wallet/passes/{ptype}/{serial}"),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "read back pass: {status} {body}");
    let mut edited = body["passJson"].clone();
    edited["logoText"] = json!("Edited Cafe");

    // ...edit it...
    let (status, _) = call(
        &state,
        json_request(
            "PATCH",
            &format!("/api/v1/wallet/passes/{ptype}/{serial}"),
            Some(&admin),
            edited,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, body) = call(
        &state,
        get(
            &format!("/api/v1/wallet/passes/{ptype}/{serial}"),
            Some(&admin),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["passJson"]["logoText"], "Edited Cafe");

    // A mismatched serialNumber in the URL vs body is refused.
    let (status, _) = call(
        &state,
        json_request(
            "PATCH",
            &format!("/api/v1/wallet/passes/{ptype}/other-serial"),
            Some(&admin),
            body["passJson"].clone(),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // ...and delete it; downloads then 404.
    let (status, _) = call(
        &state,
        raw_request(
            "DELETE",
            &format!("/api/v1/wallet/passes/{ptype}/{serial}"),
            Some(&admin),
            "",
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _, _) = call_bytes(
        &state,
        get(
            &format!("/api/v1/wallet/pass/{ptype}/{serial}.pkpass"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
