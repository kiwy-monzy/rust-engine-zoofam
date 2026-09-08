//! Session + password-reset models — the rows that back the refresh-token
//! family and the email-based password reset flow.

use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::Serialize;

use crate::schema::{gateway_password_resets, gateway_sessions};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_sessions)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub refresh_token_hash: String,
    pub device: Option<String>,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    pub created_at: NaiveDateTime,
    pub last_seen_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = gateway_sessions)]
pub struct NewSession<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub refresh_token_hash: &'a str,
    pub device: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub ip: Option<&'a str>,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = gateway_sessions)]
pub struct SessionTouch {
    pub last_seen_at: NaiveDateTime,
    pub ip: Option<Option<String>>,
    pub user_agent: Option<Option<String>>,
    pub device: Option<Option<String>>,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_password_resets)]
pub struct PasswordReset {
    pub id: String,
    pub user_id: String,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = gateway_password_resets)]
pub struct NewPasswordReset<'a> {
    pub id: &'a str,
    pub user_id: &'a str,
    pub token_hash: &'a str,
    pub expires_at: NaiveDateTime,
}
