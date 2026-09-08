// Fleet routes — Bolt login (vault-backed session), in-memory poller tile
// generation, and the marine data cookie viewer. No persistence: live fleet
// data lives only in the gateway-fleet process-wide cache (`gateway_fleet::LIVE`).
pub mod gateway;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn fleet_routes() -> Router<AppState> {
    Router::new()
        // Vault: stored credentials (Bolt session, MarineTraffic cookie, etc).
        .route("/fleet/vault", get(gateway::vault_list))
        .route("/fleet/vault", post(gateway::vault_put))
        // Bolt login flow (phone → OTP → session stored in the vault).
        .route("/fleet/bolt", get(gateway::bolt_list))
        .route(
            "/fleet/bolt/login/start",
            post(gateway::bolt_login_start),
        )
        .route(
            "/fleet/bolt/login/confirm",
            post(gateway::bolt_login_confirm),
        )
        .route("/fleet/bolt/status", get(gateway::bolt_status))
        // MarineTraffic cookie (the one the poller uses to bypass the Cloudflare
        // warm-up). POST so it can be replaced via the Fleet page.
        .route(
            "/fleet/marine/cookie",
            get(gateway::marine_cookie).post(gateway::marine_cookie_set),
        )
        // MVT / GeoJSON tiles straight from the in-memory live cache.
        //   /fleet/tiles/{source}/{z}/{x}/{y}
        //   /fleet/tiles/{source}/{z}/{x}/{y}.json
        .route(
            "/fleet/tiles/:source/:z/:x/*y",
            get(gateway::tile_handler),
        )
        // Aggregate snapshot (counts, sources, H3 cells) for the Fleet page.
        .route("/fleet/summary", get(gateway::summary))
}
