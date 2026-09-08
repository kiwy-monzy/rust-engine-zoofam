//! The Redis fleet event bus.
//!
//! The poller writes the latest H3 cell snapshot here; the `/api/fleet/live`
//! endpoint reads it back. Redis is the shared, fast hop between the two so the
//! API never recomputes the grid on the request path, and so a second gateway
//! instance would see the same snapshot. It is a cache/bus, not a store — the
//! authoritative rows stay in the database.

use anyhow::{Context, Result};
use redis::AsyncCommands;

/// Key holding the newest fleet cell FeatureCollection (JSON).
const CELLS_KEY: &str = "fleet:cells";
/// Channel a snapshot is published on, for anything that wants to subscribe.
const CELLS_CHANNEL: &str = "fleet:events";
/// Snapshots expire so a dead poller does not leave a stale grid up forever.
const TTL_SECS: u64 = 120;

#[derive(Clone)]
pub struct Bus {
    client: redis::Client,
}

impl Bus {
    pub fn connect(url: &str) -> Result<Self> {
        let client = redis::Client::open(url).context("opening Redis client")?;
        Ok(Bus { client })
    }

    async fn conn(&self) -> Result<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .context("connecting to Redis")
    }

    /// Store the latest cell snapshot and announce it.
    pub async fn publish_cells(&self, geojson: &str) -> Result<()> {
        let mut c = self.conn().await?;
        let _: () = c.set_ex(CELLS_KEY, geojson, TTL_SECS).await?;
        // Best-effort notify; subscribers are optional.
        let _: Result<i64, _> = c.publish(CELLS_CHANNEL, geojson).await;
        Ok(())
    }

    /// The latest cell snapshot, if the poller has written one recently.
    pub async fn cells(&self) -> Result<Option<String>> {
        let mut c = self.conn().await?;
        let v: Option<String> = c.get(CELLS_KEY).await?;
        Ok(v)
    }

    /// Store an arbitrary JSON string under a key (e.g. the SGR stations blob),
    /// with the same expiry as the cell snapshot.
    pub async fn set(&self, key: &str, val: &str) -> Result<()> {
        let mut c = self.conn().await?;
        let _: () = c.set_ex(key, val, TTL_SECS).await?;
        Ok(())
    }

    /// Read a key set with [`Self::set`].
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let mut c = self.conn().await?;
        Ok(c.get(key).await?)
    }

    /// A quick liveness check for the status page.
    pub async fn ping(&self) -> Result<()> {
        let mut c = self.conn().await?;
        let _: String = redis::cmd("PING").query_async(&mut c).await?;
        Ok(())
    }
}
