//! User-profile model — extends `gateway_users` with the display fields and
//! a pointer to the current avatar file (which lives in the `user_files` table).

use chrono::NaiveDateTime;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::gateway_profiles;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize)]
#[diesel(table_name = gateway_profiles, primary_key(user_id))]
pub struct Profile {
    pub user_id: String,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub avatar_file_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = gateway_profiles)]
pub struct NewProfile<'a> {
    pub user_id: &'a str,
    pub display_name: Option<&'a str>,
    pub phone: Option<&'a str>,
    pub location: Option<&'a str>,
    pub bio: Option<&'a str>,
    pub avatar_file_id: Option<i32>,
}

#[derive(Debug, Deserialize, AsChangeset)]
#[diesel(table_name = gateway_profiles)]
pub struct UpdateProfile {
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub bio: Option<String>,
    pub avatar_file_id: Option<Option<i32>>,
}
