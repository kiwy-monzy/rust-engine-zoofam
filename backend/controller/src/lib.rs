pub mod bootstrap;
pub mod crm;
pub mod dmc;
pub mod erp;
pub mod mailer;
pub mod maps;
pub mod marketplace;
pub mod permissions;
pub mod profile;
pub mod release;
pub mod roles;
pub mod search;
pub mod seed;
pub mod sessions;
pub mod storage;
pub mod subscription;
pub mod support;
pub mod system;
pub mod uploads;
pub mod users;
pub mod validate;
pub mod thirdparty;

pub use storage as file_storage;

use db::{DbError, DbPool};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Auth(#[from] auth::AuthError),
    #[error(transparent)]
    Wallet(#[from] applewallet::WalletError),
    #[error("no {0} with that id")]
    NotFound(&'static str),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Invalid(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    SessionEnded(&'static str),
    #[error("payment failed: {0}")]
    PaymentFailed(String),
    #[error("clickpesa error: {0}")]
    Clickpesa(String),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        use diesel::result::{DatabaseErrorKind, Error as E};
        match e {
            E::NotFound => Error::NotFound("record"),
            E::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                tracing::debug!(message = %info.message(), "unique violation surfaced as a conflict");
                Error::Conflict("a record with those details already exists".into())
            }
            other => Error::Db(DbError::Query(other)),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Controllers {
    pub pool: DbPool,
}

impl Controllers {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}
