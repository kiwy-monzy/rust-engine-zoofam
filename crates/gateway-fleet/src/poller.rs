//! App poller — populates the **in-memory** live cache. No DB writes.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

use db::DbPool;
use fleet::Area;
use fleet::{grid, sgr};

use crate::bus::Bus;
use crate::live::LiveFleet;
use crate::markers::to_row;
use crate::store_live;

const LAT: f64 = -6.83;
const LNG: f64 = 39.30;
const RES: u8 = 6;

/// State we remember between polls so a source that just isn't configured
/// once does not produce a fresh error every 60 s. We only re-log when the
/// "needs operator" flag actually flips, so the operator sees:
///
///   - first poll after boot, Bolt not signed in:    one warning
///   - admin signs in:                                one info line
///   - next 99 polls:                                  silence
///   - session expires / cookie is rejected:          one warning, again
///
/// Other (transient) errors are logged at debug.
#[derive(Default)]
struct OperatorNotices {
    inner: Mutex<HashSet<&'static str>>,
}

impl OperatorNotices {
    fn new() -> Self {
        Self::default()
    }

    /// Returns true the first time `source_id` is reported as needing an
    /// operator, and false on every subsequent poll until the source recovers.
    fn record(&self, source_id: &'static str) -> bool {
        let mut g = self.inner.lock().expect("operator_notices poisoned");
        g.insert(source_id)
    }

    /// Call when a source starts producing data again, so a future failure
    /// is announced.
    fn clear(&self, source_id: &'static str) {
        let mut g = self.inner.lock().expect("operator_notices poisoned");
        g.remove(&source_id);
    }
}

static NOTICES: std::sync::LazyLock<OperatorNotices> =
    std::sync::LazyLock::new(OperatorNotices::new);

/// Spawn the poller. `bus` is optional on Windows — when `None` we still poll
/// and update the in-memory cache, but skip Redis publishing.
pub fn spawn(pool: DbPool, bus: Option<Bus>, period_secs: u64) {
    if period_secs == 0 {
        return;
    }
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(period_secs.max(5)));
        loop {
            tick.tick().await;
            if let Err(e) = run_once(&pool, bus.as_ref()).await {
                tracing::error!(error = %e, "fleet poller tick failed");
            }
        }
    });
}

async fn run_once(pool: &DbPool, bus: Option<&Bus>) -> anyhow::Result<()> {
    let area = Area { lat: LAT, lng: LNG };
    let mut bolt_rows: Vec<crate::BoltVehicle> = Vec::new();
    let mut marine_rows: Vec<crate::BoltVehicle> = Vec::new();
    let mut flight_rows: Vec<crate::BoltVehicle> = Vec::new();

    for source in fleet::all() {
        // Look up the credential up front. Sources that don't need one (Flights,
        // SGR) return `None` and we just call them. Sources that do need one
        // (Bolt, Marine) we skip *silently* when the vault is empty — the
        // operator-notice tracker logs a one-time hint when the state flips.
        let secret: Option<String> = match source.credential().vault_key() {
            None => None,
            Some((service, name)) => match load_vault_secret(pool, service, name).await {
                Some(s) => Some(s),
                None => {
                    if NOTICES.record(source.id()) {
                        let what = bolt_or_marine_hint(source.id());
                        let sid = source.id();
                        tracing::info!(
                            source = sid,
                            vault = %format!("{service}/{name}"),
                            "fleet: {sid} has no stored credential yet ({what}); the source will be skipped until it is configured — visit the Fleet page to sign in"
                        );
                    }
                    continue;
                }
            },
        };

        match source.fetch(area, secret.as_deref()).await {
            Ok(catch) => {
                NOTICES.clear(source.id());
                for v in &catch.vehicles {
                    let row = to_row(v);
                    match source.id() {
                        "bolt" => bolt_rows.push(row),
                        "marine" => marine_rows.push(row),
                        _ => flight_rows.push(row),
                    }
                }
            }
            Err(fleet::Error::MissingCredential { source_id, what }) => {
                // The vault had a key, but it pointed to something Bolt wouldn't
                // accept (expired session, bad cookie). The hint matches the
                // pre-poll warning so the operator only sees one line per state.
                if NOTICES.record(source_id) {
                    tracing::warn!(source = source_id, "{source_id} needs a credential: {what}");
                }
            }
            Err(fleet::Error::Unauthorized { source_id, detail }) => {
                if NOTICES.record(source_id) {
                    tracing::warn!(
                        source = source_id,
                        "{source_id} rejected the stored credential: {detail}"
                    );
                }
            }
            Err(fleet::Error::Blocked { source_id, status }) => {
                if NOTICES.record(source_id) {
                    tracing::warn!(
                        source = source_id,
                        "{source_id} is blocking automated access (HTTP {status})"
                    );
                }
            }
            Err(e) => {
                // Transient / retryable — debug only, no operator action.
                tracing::debug!(source = source.id(), error = %e, "fleet source fetch failed");
            }
        }
    }

    let snapshot = LiveFleet::from_poll(bolt_rows, marine_rows, flight_rows);
    store_live(snapshot);

    if let Some(b) = bus {
        if let Some(snap) = crate::load_live() {
            let points: Vec<(f64, f64)> = snap
                .bolt
                .iter()
                .chain(snap.marine.iter())
                .chain(snap.flights.iter())
                .map(|v| (v.lat, v.lng))
                .collect();
            let counts = grid::bin_counts(&points, RES);
            let geojson = grid::cells_geojson(&counts).to_string();
            if let Err(e) = b.publish_cells(&geojson).await {
                tracing::debug!(error = %e, "fleet poller publish_cells failed");
            }
            match sgr::public::stations().await {
                Ok(v) => {
                    let _ = b.set("fleet:sgr", &v.to_string()).await;
                }
                Err(e) => tracing::debug!(error = %e, "fleet poller sgr fetch failed"),
            }
        }
    }
    Ok(())
}

fn bolt_or_marine_hint(source_id: &str) -> &'static str {
    match source_id {
        "bolt" => "a phone sign-in",
        "marine" => "a MarineTraffic clearance cookie",
        _ => "the source's credential",
    }
}

async fn load_vault_secret(pool: &DbPool, service: &str, name: &str) -> Option<String> {
    let pool = pool.clone();
    let svc = service.to_string();
    let nm = name.to_string();
    tokio::task::spawn_blocking(move || crate::db_ext::vault_secret(&pool, &svc, &nm))
        .await
        .ok()
        .and_then(|r| r.ok())
        .flatten()
}
