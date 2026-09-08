use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use applewallet::{ApnsEnvironment, PassKit, WalletConfig};
use auth::Jwt;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use routes::openapi::swagger_ui_custom as swagger_ui;
use routes::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gateway=debug,routes=debug,tower_http=debug".into()),
        )
        .init();

    let pool = db::create_pool()?;
    let applied = db::run_migrations(&pool)?;
    for name in &applied {
        tracing::info!("applied migration {name}");
    }
    controller::bootstrap::ensure_admin(&pool)?;

    // Initialise the SMTP mailer from environment. Safe to call early; the
    // mailer only needs SMTP_* to be set for real outbound mail.
    controller::mailer::init();

    // Backfill bbox for existing features (one-time, no-ops once done)
    {
        let controllers = controller::Controllers::new(pool.clone());
        match controllers.backfill_feature_bboxes() {
            Ok(n) if n > 0 => tracing::info!("backfilled bbox for {n} features"),
            Ok(_) => {}
            Err(e) => tracing::warn!("bbox backfill failed: {e}"),
        }
    }

    // Seed map data (idempotent — skips existing layers)
    {
        let controllers = controller::Controllers::new(pool.clone());
        match controller::seed::seed_map_data(&controllers) {
            Ok(result) => {
                if result.sources > 0 || result.layers > 0 || result.features > 0 {
                    tracing::info!(
                        "seeded map data: {} sources, {} layers, {} styles, {} features ({} skipped)",
                        result.sources, result.layers, result.styles, result.features, result.skipped
                    );
                }
                if !result.errors.is_empty() {
                    for e in &result.errors {
                        tracing::warn!("seed error: {e}");
                    }
                }
            }
            Err(e) => tracing::warn!("map seed failed: {e}"),
        }
    }

    // Ensure storage directory exists
    let storage_dir = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "assets/storage".into());
    std::fs::create_dir_all(&storage_dir).ok();
    tracing::info!("storage directory: {storage_dir}");

    // Initialize Tantivy search index.
    let search_index = {
        let index_dir = std::env::var("SEARCH_DIR").unwrap_or_else(|_| "assets/search_index".into());
        let controllers = controller::Controllers::new(pool.clone());
        match controller::search::init_search(&controllers, &std::path::PathBuf::from(&index_dir)) {
            Ok(idx) => Some(idx),
            Err(e) => {
                tracing::warn!("search index init failed: {e}");
                None
            }
        }
    };

    let wallet = build_wallet(pool.clone());

    // Fleet Redis bus — optional on Windows/dev. When REDIS_URL is unset we
    // still run the gateway; endpoints fall back to DB + live_cells.
    let bus_opt: Option<gateway_fleet::bus::Bus> = std::env::var("REDIS_URL")
        .ok()
        .filter(|s| !s.is_empty())
        .and_then(|url| match gateway_fleet::bus::Bus::connect(&url) {
            Ok(b) => {
                tracing::info!("fleet Redis bus at {url}");
                Some(b)
            }
            Err(e) => {
                tracing::warn!("fleet Redis bus unavailable ({e}) — fleet will use DB fallback");
                None
            }
        });

    let period_secs: u64 = std::env::var("FLEET_POLL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    let state = AppState::new(pool.clone(), Jwt::from_env())
        .with_wallet(wallet)
        .with_search(search_index)
        .with_bus(bus_opt.clone());

    // Background poller: walks `fleet::all()` (Bolt, MarineTraffic, FlightRadar24, SGR)
    // on `FLEET_POLL_SECS`, writes to gateway_bolt_vehicles / marine_vessels / flights
    // via `gateway-fleet::db_ext`, then bins to H3 and publishes to Redis when present.
    // Works without Redis (Windows) — cells are recomputed per request.
    gateway_fleet::poller::spawn(pool.clone(), bus_opt.clone(), period_secs);
    if period_secs > 0 {
        tracing::info!(
            "fleet poller -> every {period_secs}s (Redis {bus_label})",
            bus_label = if bus_opt.is_some() {
                "on"
            } else {
                "off (DB fallback)"
            }
        );
    }

    let mut app = routes::app(state);

    // Serve Swagger UI and OpenAPI schema
    app = app.merge(swagger_ui());

    // Serve the admin SPA from disk when it has been built (frontend/dist).
    // API paths keep their JSON 404; everything else falls back to index.html
    // so deep links like /users survive a refresh.
    let static_dir =
        PathBuf::from(std::env::var("STORAGE_DIR").unwrap_or_else(|_| "frontend/dist".into()));
    // fallback to legacy location ../frontend/dist if app/frontend/dist not found (supports both layouts)
    let static_dir = if index_html(&static_dir).is_some() {
        static_dir
    } else {
        PathBuf::from("../frontend/dist")
    };
    if index_html(&static_dir).is_some() {
        let shown = static_dir.display().to_string();
        let root = Arc::new(static_dir);
        app = app.fallback(get(move |req: Request| {
            let root = root.clone();
            async move { serve_spa(root, req) }
        }));
        tracing::info!("serving admin SPA from {shown}");
    } else {
        tracing::info!(
            "no SPA at {} (run `npm run build` in frontend/ or app/frontend/) - API only",
            static_dir.display()
        );
    }

    log_registered_routes();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| format!("invalid HOST:PORT {host}:{port}: {e}"))?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("listening on http://{addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

// ------------------------------------------------------------ static SPA --

fn index_html(root: &Path) -> Option<PathBuf> {
    let f = root.join("index.html");
    f.is_file().then_some(f)
}

/// File server with SPA semantics: try the literal file, else hand back
/// index.html — except under /api, which must always stay a JSON 404.
fn serve_spa(root: Arc<PathBuf>, req: Request) -> Response {
    let path = req.uri().path();
    if path.starts_with("/api/") || path == "/api" {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "no such api route" })),
        )
            .into_response();
    }

    // Strip the leading '/', reject traversal, and serve real files directly.
    let rel = path.trim_start_matches('/');
    let rel = percent_decode(rel);
    if !rel.is_empty() && !rel.contains("..") {
        if let Some(bytes) = std::fs::read(root.join(&rel)).ok() {
            if let Some(mime) = mime_of(&rel) {
                return ([(header::CONTENT_TYPE, mime)], bytes).into_response();
            }
        }
    }
    match std::fs::read(index_html(&root).unwrap_or_default()) {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, "text/html; charset=utf-8"),
                (header::CACHE_CONTROL, "no-store, no-cache, must-revalidate"),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

fn percent_decode(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn mime_of(path: &str) -> Option<&'static str> {
    let ext = path.rsplit('.').next()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript",
        "css" => "text/css",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "json" | "map" => "application/json",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        _ => return None,
    })
}

// ---------------------------------------------------------------- wallet --

/// Build the Apple Wallet engine from env. Missing signing material disables
/// the module (routes answer 503) rather than refusing to boot.
fn build_wallet(pool: db::DbPool) -> Option<Arc<PassKit>> {
    let base_url = std::env::var("WALLET_BASE_URL")
        .ok()
        .filter(|s| !s.is_empty());
    let base_url = base_url.unwrap_or_else(|| {
        // The QR codes and webServiceURL must resolve from the phone, so the
        // default is the LAN IP of this machine, not loopback.
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        format!("http://{}:{port}", primary_lan_ip())
    });

    let config = WalletConfig {
        pass_type_id: std::env::var("WALLET_PASS_TYPE_ID")
            .unwrap_or_else(|_| "pass.tz.ticketevent.pass.sample".to_string()),
        team_id: std::env::var("WALLET_TEAM_ID").unwrap_or_else(|_| "29S6Z4Y4MS".to_string()),
        priv_dir: PathBuf::from(std::env::var("WALLET_PRIV_DIR").unwrap_or_else(|_| "assets/priv".into())),
        assets_dir: std::env::var("WALLET_ASSETS_DIR")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from),
        web_service_path: "/api/v1/wallet/v1".to_string(),
        apns_environment: match std::env::var("WALLET_APNS_ENV").as_deref() {
            Ok("sandbox") => ApnsEnvironment::Sandbox,
            _ => ApnsEnvironment::Production,
        },
        base_url,
    };

    match PassKit::with_store(config, Arc::new(controller::dmc::WalletDbStore::new(pool))) {
        Ok(kit) => {
            tracing::info!(
                "wallet ready — passes at {}, APNs push {}",
                kit.config.base_url,
                if kit.apns_ready() {
                    "enabled"
                } else {
                    "disabled (no usable client cert)"
                }
            );
            Some(Arc::new(kit))
        }
        Err(e) => {
            tracing::warn!(
                "wallet disabled ({e}); place the Pass Type ID certificate, key and \
                 WWDR intermediate in WALLET_PRIV_DIR to enable it"
            );
            None
        }
    }
}

/// Best-effort primary LAN address: ask the OS which local interface a UDP
/// socket to a public address would use. No packets are actually sent.
fn primary_lan_ip() -> String {
    std::net::UdpSocket::bind(("0.0.0.0", 0))
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
    tracing::info!("shutting down");
}

fn log_registered_routes() {
    let routes = &[
        ("GET", "/api/v1/health"),
        ("POST", "/api/v1/auth/register"),
        ("POST", "/api/v1/auth/login"),
        ("POST", "/api/v1/auth/refresh"),
        ("POST", "/api/v1/auth/password-reset/request"),
        ("POST", "/api/v1/auth/password-reset/confirm"),
        ("GET", "/api/v1/auth/me"),
        ("POST", "/api/v1/auth/logout"),
        ("POST", "/api/v1/auth/logout-all"),
        ("GET", "/api/v1/auth/profile"),
        ("PATCH", "/api/v1/auth/profile"),
        ("POST", "/api/v1/auth/profile/avatar"),
        ("POST", "/api/v1/auth/profile/avatar/pick"),
        ("GET", "/api/v1/auth/profile/avatars"),
        ("POST", "/api/v1/auth/profile/password"),
        ("GET", "/api/v1/auth/profile/sessions"),
        ("DELETE", "/api/v1/auth/profile/sessions/:id"),
        ("GET", "/api/v1/users"),
        ("POST", "/api/v1/users"),
        ("GET", "/api/v1/users/search"),
        ("GET", "/api/v1/users/:id"),
        ("PATCH", "/api/v1/users/:id"),
        ("DELETE", "/api/v1/users/:id"),
        ("POST", "/api/v1/users/:id/roles/:role_id"),
        ("DELETE", "/api/v1/users/:id/roles/:role_id"),
        ("GET", "/api/v1/roles"),
        ("POST", "/api/v1/roles"),
        ("GET", "/api/v1/roles/:id"),
        ("PATCH", "/api/v1/roles/:id"),
        ("DELETE", "/api/v1/roles/:id"),
        ("POST", "/api/v1/roles/:id/permissions/:permission_id"),
        ("DELETE", "/api/v1/roles/:id/permissions/:permission_id"),
        ("GET", "/api/v1/permissions"),
        ("POST", "/api/v1/permissions"),
        ("PATCH", "/api/v1/permissions/:id"),
        ("DELETE", "/api/v1/permissions/:id"),
        ("GET", "/api/v1/system"),
        ("PATCH", "/api/v1/system"),
        ("GET", "/api/v1/system/version"),
        ("GET", "/api/v1/system/releases"),
        ("POST", "/api/v1/system/releases"),
        ("GET", "/api/v1/system/releases/latest"),
        ("GET", "/api/v1/system/releases/check/:version"),
        ("DELETE", "/api/v1/system/releases/:id"),
        ("GET", "/api/v1/system/releases/:id/download"),
        ("GET", "/api/v1/support"),
        ("POST", "/api/v1/support"),
        ("GET", "/api/v1/support/users"),
        ("POST", "/api/v1/support/:id/close"),
        ("GET", "/api/v1/maps/*"),
        ("POST", "/api/v1/uploads/*"),
        ("GET", "/api/v1/storage/*"),
        ("GET", "/api/v1/fleet/*"),
        ("GET", "/api/v1/erp/*"),
        ("GET", "/api/v1/crm/*"),
        ("GET", "/api/v1/dmc/*"),
        ("GET", "/api/v1/marketplace/*"),
        ("GET", "/api/v1/website/*"),
        ("GET", "/api/v1/search"),
        ("POST", "/api/v1/search/reindex"),
        ("GET", "/api/v1/wallet/pass/:pass_type/:serial_pkpass"),
        ("POST", "/api/v1/wallet/v1/devices/:device_id/registrations/:pass_type/:serial"),
        ("DELETE", "/api/v1/wallet/v1/devices/:device_id/registrations/:pass_type/:serial"),
        ("GET", "/api/v1/wallet/v1/devices/:device_id/registrations/:pass_type"),
        ("GET", "/api/v1/wallet/v1/passes/:pass_type/:serial"),
        ("POST", "/api/v1/wallet/v1/log"),
        ("GET", "/api/v1/routes"),
        ("GET", "/health"),
        ("GET", "/sprites/*path"),
        ("GET", "/glyphs/*path"),
    ];

    tracing::info!("=== REGISTERED ROUTES ({} total) ===", routes.len());
    for (method, path) in routes {
        tracing::info!("  {method:6} {path}");
    }
    tracing::info!("==================================");
}
