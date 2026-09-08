//! Session lifecycle — one row in `gateway_sessions` per login. The JWT itself
//! remains stateless, but the refresh-token family and the user-agent are
//! persisted so /auth/profile/sessions can list and revoke them, and so
//! /auth/refresh can verify the family is still live.

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;

use auth::{Claims, RefreshClaims};
use db::{conn, DbPool};
use models::schema::{gateway_revoked_tokens, gateway_sessions, gateway_users};
use models::session::{NewSession, Session};
use models::NewRevokedToken;

use crate::{Error, Result};

pub fn validate(pool: &DbPool, claims: &Claims) -> Result<()> {
    let mut c = conn(pool)?;

    let (is_active, token_version): (bool, i32) = gateway_users::table
        .find(&claims.sub)
        .select((gateway_users::is_active, gateway_users::token_version))
        .first(&mut c)
        .map_err(|_| Error::SessionEnded("the account no longer exists"))?;

    if !is_active {
        return Err(Error::SessionEnded("the account is disabled"));
    }
    if token_version != claims.ver {
        return Err(Error::SessionEnded("signed out everywhere; sign in again"));
    }

    let revoked: bool = diesel::select(diesel::dsl::exists(
        gateway_revoked_tokens::table.filter(gateway_revoked_tokens::jti.eq(&claims.jti)),
    ))
    .get_result(&mut c)?;
    if revoked {
        return Err(Error::SessionEnded("this session was signed out"));
    }

    // The refresh-token family is gone, so the access token is dead even
    // though the JWT itself is still cryptographically valid.
    let session_live: bool = diesel::select(diesel::dsl::exists(
        gateway_sessions::table
            .filter(gateway_sessions::id.eq(&claims.sid))
            .filter(gateway_sessions::revoked_at.is_null()),
    ))
    .get_result(&mut c)?;
    if !session_live {
        return Err(Error::SessionEnded("this session was signed out"));
    }
    Ok(())
}

pub fn revoke(pool: &DbPool, claims: &Claims) -> Result<()> {
    let expires_at = DateTime::from_timestamp(claims.exp, 0)
        .unwrap_or_else(Utc::now)
        .naive_utc();
    let record = NewRevokedToken {
        jti: claims.jti.clone(),
        user_id: claims.sub.clone(),
        expires_at,
    };

    let mut c = conn(pool)?;
    diesel::insert_into(gateway_revoked_tokens::table)
        .values(&record)
        .on_conflict_do_nothing()
        .execute(&mut c)?;
    diesel::update(gateway_sessions::table.find(&claims.sid))
        .set(gateway_sessions::revoked_at.eq(Some(Utc::now().naive_utc())))
        .execute(&mut c)?;
    purge_expired(&mut c)?;
    Ok(())
}

pub fn revoke_all(pool: &DbPool, user_id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::update(gateway_users::table.find(user_id))
        .set(gateway_users::token_version.eq(gateway_users::token_version + 1))
        .execute(&mut c)?;
    diesel::update(gateway_sessions::table.filter(gateway_sessions::user_id.eq(user_id)))
        .set(gateway_sessions::revoked_at.eq(Some(Utc::now().naive_utc())))
        .execute(&mut c)?;
    Ok(())
}

pub fn revoke_by_id(pool: &DbPool, user_id: &str, session_id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let n = diesel::update(
        gateway_sessions::table
            .filter(gateway_sessions::id.eq(session_id))
            .filter(gateway_sessions::user_id.eq(user_id)),
    )
    .set(gateway_sessions::revoked_at.eq(Some(Utc::now().naive_utc())))
    .execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("session"));
    }
    Ok(())
}

pub fn list_for_user(pool: &DbPool, user_id: &str) -> Result<Vec<Session>> {
    let mut c = conn(pool)?;
    let rows: Vec<Session> = gateway_sessions::table
        .filter(gateway_sessions::user_id.eq(user_id))
        .order(gateway_sessions::last_seen_at.desc())
        .load(&mut c)?;
    Ok(rows)
}

pub fn find_by_id(pool: &DbPool, session_id: &str) -> Result<Option<Session>> {
    let mut c = conn(pool)?;
    Ok(gateway_sessions::table
        .filter(gateway_sessions::id.eq(session_id))
        .first::<Session>(&mut c)
        .optional()?)
}

pub fn create(
    pool: &DbPool,
    id: &str,
    user_id: &str,
    refresh_token_hash: &str,
    device: Option<&str>,
    user_agent: Option<&str>,
    ip: Option<&str>,
    expires_at: NaiveDateTime,
) -> Result<()> {
    let mut c = conn(pool)?;
    let new = NewSession {
        id,
        user_id,
        refresh_token_hash,
        device,
        user_agent,
        ip,
        expires_at,
    };
    diesel::insert_into(gateway_sessions::table)
        .values(&new)
        .execute(&mut c)?;
    Ok(())
}

pub fn find_by_refresh_hash(pool: &DbPool, refresh_token_hash: &str) -> Result<Option<Session>> {
    let mut c = conn(pool)?;
    Ok(gateway_sessions::table
        .filter(gateway_sessions::refresh_token_hash.eq(refresh_token_hash))
        .first::<Session>(&mut c)
        .optional()?)
}

pub fn rotate_refresh_hash(pool: &DbPool, session_id: &str, new_hash: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::update(gateway_sessions::table.find(session_id))
        .set((
            gateway_sessions::refresh_token_hash.eq(new_hash),
            gateway_sessions::last_seen_at.eq(Utc::now().naive_utc()),
        ))
        .execute(&mut c)?;
    Ok(())
}

pub fn touch(
    pool: &DbPool,
    session_id: &str,
    ip: Option<&str>,
    user_agent: Option<&str>,
) -> Result<()> {
    let mut c = conn(pool)?;
    let now = Utc::now().naive_utc();
    if let (Some(ip), Some(ua)) = (ip, user_agent) {
        diesel::update(gateway_sessions::table.find(session_id))
            .set((
                gateway_sessions::last_seen_at.eq(now),
                gateway_sessions::ip.eq(Some(ip)),
                gateway_sessions::user_agent.eq(Some(ua)),
            ))
            .execute(&mut c)?;
    } else {
        diesel::update(gateway_sessions::table.find(session_id))
            .set(gateway_sessions::last_seen_at.eq(now))
            .execute(&mut c)?;
    }
    Ok(())
}

pub fn validate_refresh(
    pool: &DbPool,
    claims: &RefreshClaims,
    refresh_hash: &str,
) -> Result<Session> {
    let s =
        find_by_id(pool, &claims.sid)?.ok_or(Error::SessionEnded("refresh session not found"))?;
    if s.revoked_at.is_some() {
        return Err(Error::SessionEnded("this session was signed out"));
    }
    if s.user_id != claims.sub {
        return Err(Error::SessionEnded("refresh does not belong to this user"));
    }
    if s.refresh_token_hash != refresh_hash {
        return Err(Error::SessionEnded("refresh token rotated or replaced"));
    }
    let exp: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(s.expires_at, Utc);
    if exp < Utc::now() {
        return Err(Error::SessionEnded("refresh token expired"));
    }
    Ok(s)
}

fn purge_expired(c: &mut db::DbConn) -> Result<()> {
    diesel::delete(
        gateway_revoked_tokens::table
            .filter(gateway_revoked_tokens::expires_at.lt(Utc::now().naive_utc())),
    )
    .execute(c)?;
    Ok(())
}

// ---------------------------------------------------------- password reset --

use models::schema::gateway_password_resets;
use models::session::{NewPasswordReset, PasswordReset};

pub fn create_reset(
    pool: &DbPool,
    id: &str,
    user_id: &str,
    token_hash: &str,
    expires_at: NaiveDateTime,
) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::insert_into(gateway_password_resets::table)
        .values(NewPasswordReset {
            id,
            user_id,
            token_hash,
            expires_at,
        })
        .execute(&mut c)?;
    Ok(())
}

pub fn find_reset_by_hash(pool: &DbPool, token_hash: &str) -> Result<Option<PasswordReset>> {
    let mut c = conn(pool)?;
    Ok(gateway_password_resets::table
        .filter(gateway_password_resets::token_hash.eq(token_hash))
        .first::<PasswordReset>(&mut c)
        .optional()?)
}

pub fn consume_reset(pool: &DbPool, reset_id: &str) -> Result<()> {
    let mut c = conn(pool)?;
    diesel::update(gateway_password_resets::table.find(reset_id))
        .set(gateway_password_resets::used_at.eq(Some(Utc::now().naive_utc())))
        .execute(&mut c)?;
    Ok(())
}
