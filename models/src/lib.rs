pub mod schema;
pub mod fleet_events;
pub mod erp;
pub mod crm;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use schema::{
    gateway_permissions, gateway_releases, gateway_revoked_tokens, gateway_role_permissions,
    gateway_roles, gateway_system, gateway_user_roles, gateway_users, support_tickets, user_files,
    wallet_passes, wallet_registrations,
};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_users)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip)]
    pub email_lower: String,
    #[serde(skip)]
    pub password_hash: String,
    pub display_name: String,
    pub avatar_url: String,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    #[serde(skip)]
    pub token_version: i32,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = gateway_users)]
pub struct NewUser {
    pub id: String,
    pub email: String,
    pub email_lower: String,
    pub password_hash: String,
    pub display_name: String,
    pub avatar_url: String,
    pub is_active: bool,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = gateway_users)]
pub struct UserPatch {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_roles)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = gateway_roles)]
pub struct NewRole {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_permissions)]
pub struct Permission {
    pub id: i32,
    pub module: String,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = gateway_permissions)]
pub struct NewPermission {
    pub module: String,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = gateway_permissions)]
pub struct UpdatePermission {
    pub module: Option<String>,
    pub action: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = gateway_role_permissions)]
pub struct RolePermission {
    pub role_id: i32,
    pub permission_id: i32,
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = gateway_user_roles)]
pub struct UserRole {
    pub user_id: String,
    pub role_id: i32,
}

#[derive(Debug, Serialize)]
pub struct UserWithRoles {
    #[serde(flatten)]
    pub user: User,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RoleWithPermissions {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub avatar_url: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRole {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = gateway_roles)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

// ------------------------------------------------------------------ system --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_system)]
pub struct SystemSetting {
    pub id: i32,
    pub name: String,
    pub logo_url: String,
    pub version: String,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = gateway_system)]
pub struct UpdateSystem {
    pub name: Option<String>,
    pub logo_url: Option<String>,
    pub version: Option<String>,
}

// ----------------------------------------------------------------- releases --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_releases)]
pub struct Release {
    pub id: i32,
    pub version: String,
    pub platform: String,
    pub filename: String,
    pub file_size: i32,
    pub sha256: Option<String>,
    pub changelog: Option<String>,
    pub download_count: i32,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = gateway_releases)]
pub struct NewRelease {
    pub version: String,
    pub platform: String,
    pub filename: String,
    pub file_size: i32,
    pub sha256: Option<String>,
    pub changelog: Option<String>,
}

// ----------------------------------------------------------------- support --

pub const TICKET_CATEGORIES: [&str; 3] = [
    "viewer_to_admin",
    "admin_to_admin",
    "viewer_to_viewer",
];

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = support_tickets)]
pub struct SupportTicket {
    pub id: String,
    pub category: String,
    pub sender_id: String,
    pub recipient_id: Option<String>,
    pub subject: String,
    pub body: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = support_tickets)]
pub struct NewTicket {
    pub id: String,
    pub category: String,
    pub sender_id: String,
    pub recipient_id: Option<String>,
    pub subject: String,
    pub body: String,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateTicket {
    pub category: String,
    #[serde(default)]
    pub recipient_id: Option<String>,
    pub subject: String,
    pub body: String,
}

/// A ticket with human-readable sender/recipient labels for the admin UI.
#[derive(Debug, Serialize)]
pub struct TicketView {
    #[serde(flatten)]
    pub ticket: SupportTicket,
    pub sender: String,
    pub recipient: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = gateway_revoked_tokens)]
pub struct NewRevokedToken {
    pub jti: String,
    pub user_id: String,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = wallet_passes)]
#[diesel(primary_key(pass_type_id, serial_number))]
pub struct WalletPass {
    pub pass_type_id: String,
    pub serial_number: String,
    #[serde(skip)]
    pub auth_token: String,
    /// Raw `pass.json` bytes; surfaced to clients as parsed JSON.
    #[serde(skip)]
    pub pass_json: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = wallet_passes)]
pub struct NewWalletPass {
    pub pass_type_id: String,
    pub serial_number: String,
    pub auth_token: String,
    pub pass_json: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = wallet_registrations)]
#[diesel(primary_key(device_library_id, pass_type_id, serial_number))]
pub struct WalletRegistration {
    #[serde(skip_serializing)]
    pub device_library_id: String,
    pub pass_type_id: String,
    pub serial_number: String,
    #[serde(skip_serializing)]
    pub push_token: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = wallet_registrations)]
pub struct NewWalletRegistration {
    pub device_library_id: String,
    pub pass_type_id: String,
    pub serial_number: String,
    pub push_token: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = user_files)]
pub struct UserFile {
    pub id: i32,
    pub user_id: String,
    pub collection: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = user_files)]
pub struct NewUserFile {
    pub user_id: String,
    pub collection: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
