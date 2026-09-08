//! Password hashing (argon2) and reset-token hashing.

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use sha2::{Digest, Sha256};

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("the token is missing")]
    Missing,
    #[error("the token is not valid")]
    Invalid,
    #[error("the token has expired")]
    Expired,
    #[error("the email or password is wrong")]
    BadCredentials,
    #[error("this account is disabled")]
    Disabled,
    #[error("this account may not {0}")]
    Forbidden(String),
    #[error("could not hash the password: {0}")]
    Hashing(String),
}

pub type Result<T> = std::result::Result<T, AuthError>;

pub fn hash_password(plain: &str) -> Result<String> {
    let password = Argon2::default()
        .hash_password(plain.as_bytes())
        .map_err(|e| AuthError::Hashing(e.to_string()))?;
    Ok(password.to_string())
}

pub fn verify_password(plain: &str, hash: &str) -> Result<()> {
    let parsed = PasswordHash::try_from(hash).map_err(|_| AuthError::BadCredentials)?;
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .map_err(|_| AuthError::BadCredentials)
}

/// Hash an opaque reset token with SHA-256. We use SHA-256 instead of argon2
/// because the token is high-entropy (32 random bytes) — argon2 is meant to
/// slow brute force on *low-entropy* user passwords, which is irrelevant here.
pub fn hash_reset_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}
