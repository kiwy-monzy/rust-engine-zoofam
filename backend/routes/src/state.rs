use std::sync::Arc;

use auth::Jwt;
use controller::Controllers;
use db::DbPool;
use search::SearchIndex;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub jwt: Jwt,
    /// Apple Wallet engine. `None` when the signing material is missing —
    /// wallet endpoints then answer 503 instead of failing at startup.
    pub wallet: Option<Arc<applewallet::PassKit>>,
    /// Tantivy full-text search index.
    pub search: Option<Arc<SearchIndex>>,
    /// Redis event bus for fleet H3 cells / SGR. `None` on Windows or when
    /// `REDIS_URL` is unset — endpoints fall back to computing from DB.
    pub bus: Option<Arc<gateway_fleet::bus::Bus>>,
}

impl AppState {
    pub fn new(pool: DbPool, jwt: Jwt) -> Self {
        Self {
            pool,
            jwt,
            wallet: None,
            search: None,
            bus: None,
        }
    }

    pub fn with_wallet(mut self, wallet: Option<Arc<applewallet::PassKit>>) -> Self {
        self.wallet = wallet;
        self
    }

    pub fn with_search(mut self, search: Option<Arc<SearchIndex>>) -> Self {
        self.search = search;
        self
    }

    pub fn with_bus(mut self, bus: Option<gateway_fleet::bus::Bus>) -> Self {
        self.bus = bus.map(Arc::new);
        self
    }

    pub fn controllers(&self) -> Controllers {
        Controllers::new(self.pool.clone())
    }
}
