//! Profile CRUD + avatar management. The profile is a 1:1 row that extends
//! `gateway_users` (which still owns `display_name` for backwards compatibility).
//! Avatar files live in the `user_files` table under the
//! `profile_avatars` collection; this module just points at one of them via
//! `avatar_file_id`.

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use models::{
    profile::{NewProfile, Profile, UpdateProfile},
    schema::gateway_profiles,
    schema::gateway_users,
};

use crate::{Error, Result};

pub fn get(pool: &db::DbPool, user_id: &str) -> Result<Profile> {
    let mut c = db::conn(pool)?;
    let p: Option<Profile> = gateway_profiles::table
        .find(user_id)
        .first(&mut c)
        .optional()?;
    Ok(p.unwrap_or_else(|| empty_profile(user_id)))
}

fn empty_profile(user_id: &str) -> Profile {
    let now = Utc::now().naive_utc();
    Profile {
        user_id: user_id.to_string(),
        display_name: None,
        phone: None,
        location: None,
        bio: None,
        avatar_file_id: None,
        created_at: now,
        updated_at: now,
    }
}

pub fn upsert(pool: &db::DbPool, user_id: &str, patch: &UpdateProfile) -> Result<Profile> {
    let mut c = db::conn(pool)?;
    let now = Utc::now().naive_utc();
    let existing = gateway_profiles::table
        .find(user_id)
        .first::<Profile>(&mut c)
        .optional()?;

    if let Some(_) = existing {
        diesel::update(gateway_profiles::table.find(user_id))
            .set((patch, gateway_profiles::updated_at.eq(now)))
            .execute(&mut c)?;
    } else {
        // First-time write — translate the Update into a New.
        let new = NewProfile {
            user_id,
            display_name: patch.display_name.as_deref(),
            phone: patch.phone.as_deref(),
            location: patch.location.as_deref(),
            bio: patch.bio.as_deref(),
            avatar_file_id: match patch.avatar_file_id {
                Some(Some(v)) => Some(v),
                _ => None,
            },
        };
        diesel::insert_into(gateway_profiles::table)
            .values(&new)
            .execute(&mut c)?;
    }

    // Keep gateway_users.display_name in sync: prefer explicit display_name,
    // otherwise compute from first_name + last_name on the user row.
    if let Some(name) = patch.display_name.as_ref() {
        diesel::update(gateway_users::table.find(user_id))
            .set(gateway_users::display_name.eq(name))
            .execute(&mut c)?;
    } else {
        // Rebuild display_name from the user's name fields.
        let user: Option<(Option<String>, Option<String>, Option<String>)> = gateway_users::table
            .find(user_id)
            .select((
                gateway_users::first_name.nullable(),
                gateway_users::middle_name.nullable(),
                gateway_users::last_name.nullable(),
            ))
            .first(&mut c)
            .optional()?;
        if let Some((first, middle, last)) = user {
            let parts: Vec<String> = [first, middle, last]
                .into_iter()
                .flatten()
                .filter(|s| !s.is_empty())
                .collect();
            if !parts.is_empty() {
                let computed = parts.join(" ");
                diesel::update(gateway_users::table.find(user_id))
                    .set(gateway_users::display_name.eq(computed))
                    .execute(&mut c)?;
            }
        }
    }

    Ok(gateway_profiles::table.find(user_id).first(&mut c)?)
}

/// Pick a stored file (must belong to the same user) as the new avatar.
pub fn set_avatar(pool: &db::DbPool, user_id: &str, file_id: i32) -> Result<Profile> {
    use models::schema::user_files;
    let mut c = db::conn(pool)?;
    let owned: Option<i32> = user_files::table
        .filter(user_files::id.eq(file_id))
        .filter(user_files::user_id.eq(user_id))
        .filter(user_files::collection.eq("profile_avatars"))
        .select(user_files::id)
        .first(&mut c)
        .optional()?;
    let owned = owned.ok_or(Error::Invalid(
        "file is not in your profile_avatars collection".into(),
    ))?;
    let now = Utc::now().naive_utc();

    if gateway_profiles::table
        .find(user_id)
        .first::<Profile>(&mut c)
        .optional()?
        .is_some()
    {
        diesel::update(gateway_profiles::table.find(user_id))
            .set((
                gateway_profiles::avatar_file_id.eq(owned),
                gateway_profiles::updated_at.eq(now),
            ))
            .execute(&mut c)?;
    } else {
        let new = NewProfile {
            user_id,
            display_name: None,
            phone: None,
            location: None,
            bio: None,
            avatar_file_id: Some(owned),
        };
        diesel::insert_into(gateway_profiles::table)
            .values(&new)
            .execute(&mut c)?;
    }
    Ok(gateway_profiles::table.find(user_id).first(&mut c)?)
}

pub fn list_avatars(pool: &db::DbPool, user_id: &str) -> Result<Vec<models::UserFile>> {
    use models::schema::user_files;
    let mut c = db::conn(pool)?;
    let rows: Vec<models::UserFile> = user_files::table
        .filter(user_files::user_id.eq(user_id))
        .filter(user_files::collection.eq("profile_avatars"))
        .order(user_files::created_at.desc())
        .select(models::UserFile::as_select())
        .load(&mut c)?;
    Ok(rows)
}

/// Rebuild display_name from the user's first/middle/last name fields and
/// sync it to gateway_users.
pub fn sync_display_name(pool: &db::DbPool, user_id: &str) -> Result<()> {
    let mut c = db::conn(pool)?;
    let user: Option<(String, String, String)> = gateway_users::table
        .find(user_id)
        .select((
            gateway_users::first_name,
            gateway_users::middle_name,
            gateway_users::last_name,
        ))
        .first(&mut c)
        .optional()?;
    if let Some((first, middle, last)) = user {
        let parts: Vec<&str> = [&*first, &*middle, &*last]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect();
        if !parts.is_empty() {
            let computed = parts.join(" ");
            diesel::update(gateway_users::table.find(user_id))
                .set(gateway_users::display_name.eq(computed))
                .execute(&mut c)?;
        }
    }
    Ok(())
}

#[allow(dead_code)]
fn _naive_now() -> NaiveDateTime {
    Utc::now().naive_utc()
}
