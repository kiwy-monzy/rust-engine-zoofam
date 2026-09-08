pub mod crm;
pub mod dmc;
pub mod erp;
pub mod marketplace;
pub mod profile;
pub mod schema;
pub mod session;
pub mod website;

pub mod subscription;
pub use subscription::{Invoice, NewInvoice, NewPlan, NewSubscription, Plan, Subscription, UpdatePlan};

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use schema::{
    dmc_wallet_passes, dmc_wallet_registrations, gateway_permissions, gateway_releases,
    gateway_revoked_tokens, gateway_role_permissions, gateway_roles, gateway_system,
    gateway_user_roles, gateway_users, support_tickets, user_files,
};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
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
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub username: String,
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
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub username: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = gateway_users)]
pub struct UserPatch {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
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

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = gateway_permissions)]
pub struct Permission {
    pub id: i32,
    pub module: String,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Insertable, ToSchema)]
#[diesel(table_name = gateway_permissions)]
pub struct NewPermission {
    pub module: String,
    pub action: String,
    pub description: String,
}

#[derive(Debug, Deserialize, AsChangeset, ToSchema)]
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

#[derive(Debug, Serialize, ToSchema)]
pub struct UserWithRoles {
    #[serde(flatten)]
    pub user: User,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RoleWithPermissions {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUser {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub middle_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub username: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUser {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRole {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = gateway_roles)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

// ------------------------------------------------------------------ system --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = gateway_system)]
#[diesel(primary_key(key))]
pub struct SystemSetting {
    pub key: String,
    pub value: String,
    pub updated_at: chrono::NaiveDateTime,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, ToSchema)]
#[diesel(table_name = gateway_system)]
pub struct SystemSettingRow {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name = gateway_system)]
pub struct UpdateSystem {
    pub value: Option<String>,
}

// ----------------------------------------------------------------- releases --

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = gateway_releases)]
pub struct Release {
    pub id: i32,
    pub version: String,
    pub platform: String,
    pub filename: String,
    pub file_size: Option<i32>,
    pub sha256: Option<String>,
    pub changelog: Option<String>,
    pub download_count: i32,
    pub created_by: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Insertable, ToSchema)]
#[diesel(table_name = gateway_releases)]
pub struct NewRelease {
    pub version: String,
    pub platform: String,
    pub filename: String,
    pub file_size: Option<i32>,
    pub sha256: Option<String>,
    pub changelog: Option<String>,
    pub created_by: Option<String>,
}

// ----------------------------------------------------------------- support --

pub const TICKET_CATEGORIES: [&str; 3] = ["viewer_to_admin", "admin_to_admin", "viewer_to_viewer"];

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, ToSchema)]
#[diesel(table_name = support_tickets)]
pub struct SupportTicket {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub category: String,
    pub created_by: String,
    pub assigned_to: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = support_tickets)]
pub struct NewTicket {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub category: String,
    pub created_by: String,
    pub assigned_to: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub closed_at: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTicket {
    pub category: String,
    #[serde(default)]
    pub recipient_id: Option<String>,
    pub subject: String,
    pub body: String,
}

/// A ticket with human-readable sender/recipient labels for the admin UI.
#[derive(Debug, Serialize, ToSchema)]
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
#[diesel(table_name = dmc_wallet_passes)]
pub struct WalletPass {
    pub id: String,
    pub pass_type: String,
    pub serial_number: String,
    pub auth_token: String,
    pub user_id: Option<String>,
    pub data: String,
    pub is_active: bool,
    pub last_updated: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = dmc_wallet_passes)]
pub struct NewWalletPass {
    pub id: String,
    pub pass_type: String,
    pub serial_number: String,
    pub auth_token: String,
    pub user_id: Option<String>,
    pub data: String,
    pub is_active: bool,
    pub last_updated: NaiveDateTime,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = dmc_wallet_registrations)]
pub struct WalletRegistration {
    pub id: String,
    pub device_id: String,
    pub pass_type: String,
    pub serial_number: String,
    pub push_token: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = dmc_wallet_registrations)]
pub struct NewWalletRegistration {
    pub id: String,
    pub device_id: String,
    pub pass_type: String,
    pub serial_number: String,
    pub push_token: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = user_files)]
pub struct UserFile {
    pub id: i32,
    pub user_id: String,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i32,
    pub storage_path: String,
    pub collection: String,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = user_files)]
pub struct NewUserFile {
    pub user_id: String,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i32,
    pub storage_path: String,
    pub collection: String,
    pub is_public: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
