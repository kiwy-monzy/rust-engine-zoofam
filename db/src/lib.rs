use diesel::connection::SimpleConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

#[cfg(all(feature = "postgres", feature = "sqlite"))]
compile_error!("pick one backend: --features postgres or --features sqlite, not both");

#[cfg(not(any(feature = "postgres", feature = "sqlite")))]
compile_error!("pick one backend: --features postgres or --features sqlite");

#[cfg(feature = "postgres")]
pub type Connection = diesel::pg::PgConnection;
#[cfg(feature = "sqlite")]
pub type Connection = diesel::sqlite::SqliteConnection;

#[cfg(feature = "postgres")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations/postgres");
#[cfg(feature = "sqlite")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../migrations/sqlite");

#[cfg(feature = "postgres")]
pub const BACKEND: &str = "postgres";
#[cfg(feature = "sqlite")]
pub const BACKEND: &str = "sqlite";

#[cfg(feature = "postgres")]
pub const DEFAULT_URL: &str = "postgres://postgres:postgres@localhost/admin_app";
#[cfg(feature = "sqlite")]
pub const DEFAULT_URL: &str = "admin.db";

pub type DbPool = Pool<ConnectionManager<Connection>>;
pub type DbConn = PooledConnection<ConnectionManager<Connection>>;

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
    let pool = Pool::builder().max_size(pool_size(url)).build(manager)?;
    enable_foreign_keys(&pool)?;
    Ok(pool)
}

#[cfg(feature = "sqlite")]
fn pool_size(url: &str) -> u32 {
    if url.contains(":memory:") {
        1
    } else {
        8
    }
}

#[cfg(feature = "postgres")]
fn pool_size(_url: &str) -> u32 {
    8
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

#[cfg(feature = "sqlite")]
fn enable_foreign_keys(pool: &DbPool) -> Result<()> {
    let mut c = conn(pool)?;
    c.batch_execute("PRAGMA foreign_keys = ON;")?;
    Ok(())
}

#[cfg(feature = "postgres")]
fn enable_foreign_keys(_pool: &DbPool) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_in_memory_database_takes_every_migration() {
        let pool = create_pool_from(":memory:").expect("pool");
        let applied = run_migrations(&pool).expect("migrations");
        // Bump this when a migration folder is added. Note: embed_migrations!
        // may not notice a brand-new folder until this crate is rebuilt.
        assert_eq!(
            applied.len(),
            32,
            "all migrations applied (zoho_032 stub counts as a migration)"
        );
    }
}
