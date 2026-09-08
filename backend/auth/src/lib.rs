//! `auth` — password hashing + JWT issuance. Split into sub-modules:
//!
//!   password — argon2 hash/verify + reset-token hashing
//!   token    — JWT access + refresh tokens, claims, key rotation
//!
//! The public surface stays the same so the rest of the workspace compiles
//! unchanged. New `RefreshToken` and `AccessToken` types are available via
//! `auth::token::{AccessToken, RefreshToken, RefreshClaims}`.

pub mod password;
pub mod token;

pub use password::{hash_password, hash_reset_token, verify_password, AuthError as _AuthError};
pub use token::{bearer, AccessToken, AuthError, Claims, Jwt, RefreshClaims, RefreshToken};

pub type Result<T> = std::result::Result<T, AuthError>;
