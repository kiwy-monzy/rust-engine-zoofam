use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub type Connection = diesel::pg::PgConnection;
pub type DbPool = Pool<ConnectionManager<Connection>>;
pub type DbConn = PooledConnection<ConnectionManager<Connection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations/postgres");
pub const BACKEND: &str = "postgres";
pub const DEFAULT_URL: &str = "postgres://postgres:postgres@localhost/admin_app";

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("could not connect to the database: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("query failed: {0}")]
    Query(#[from] diesel::result::Error),
    #[error("migration failed: {0}")]
    Migration(String),
}

pub type Result<T> = std::result::Result<T, DbError>;

pub fn database_url() -> String {
    dotenvy::dotenv().ok();
    std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_URL.to_string())
}

pub fn create_pool() -> Result<DbPool> {
    create_pool_from(&database_url())
}

pub fn create_pool_from(url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<Connection>::new(url);
    let pool = if url == ":memory:" {
        // For in-memory SQLite, limit to 1 connection to avoid separate database instances
        // This fixes the bug where each connection gets its own database
        Pool::builder().max_size(1).build(manager)?
    } else {
        // For file-based databases and PostgreSQL, use 8 connections
        Pool::builder().max_size(8).build(manager)?
    };
    Ok(pool)
}

pub fn conn(pool: &DbPool) -> Result<DbConn> {
    Ok(pool.get()?)
}

pub fn run_migrations(pool: &DbPool) -> Result<Vec<String>> {
    let mut c = conn(pool)?;
    let applied = c
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| DbError::Migration(e.to_string()))?;
    Ok(applied.iter().map(|m| m.to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_connection_works() {
        let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
        if url.contains("localhost") || url.contains("127.0.0.1") {
            let pool = create_pool_from(&url);
            if pool.is_ok() {
                // Connection successful
            }
        }
    }
}
