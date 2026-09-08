use chrono::Utc;
use diesel::prelude::*;
use uuid::Uuid;

use db::DbConn;
use models::schema::{
    gateway_permissions, gateway_role_permissions, gateway_roles, gateway_user_roles, gateway_users,
};
use models::{CreateUser, NewUser, UpdateUser, User, UserPatch, UserWithRoles};

use crate::validate;
use crate::{Error, Result};

pub fn list(conn: &mut DbConn) -> Result<Vec<UserWithRoles>> {
    let users: Vec<User> = gateway_users::table
        .select(User::as_select())
        .order(gateway_users::created_at.desc())
        .load(conn)?;

    let mut out = Vec::with_capacity(users.len());
    for user in users {
        let roles = roles_on(conn, &user.id)?;
        out.push(UserWithRoles { user, roles });
    }
    Ok(out)
}

pub fn get(conn: &mut DbConn, id: &str) -> Result<UserWithRoles> {
    let user: User = gateway_users::table
        .find(id)
        .select(User::as_select())
        .first(conn)
        .map_err(|_| Error::NotFound("user"))?;
    let roles = roles_on(conn, &user.id)?;
    Ok(UserWithRoles { user, roles })
}

/// Bare `User` (no roles) — used by the auth flow where we just need the
/// password hash + token version.
pub fn find_by_id(conn: &mut DbConn, id: &str) -> Result<User> {
    gateway_users::table
        .find(id)
        .select(User::as_select())
        .first(conn)
        .map_err(|_| Error::NotFound("user"))
}

pub fn find_by_email(conn: &mut DbConn, email: &str) -> Result<User> {
    gateway_users::table
        .filter(gateway_users::email_lower.eq(email.trim().to_lowercase()))
        .select(User::as_select())
        .first(conn)
        .map_err(|_| Error::NotFound("user"))
}

pub fn create(conn: &mut DbConn, input: CreateUser) -> Result<User> {
    let email = validate::email(&input.email)?;
    validate::password(&input.password)?;
    validate::display_name(&input.display_name)?;
    validate::avatar_url(&input.avatar_url)?;

    let display_name = if !input.first_name.is_empty() || !input.last_name.is_empty() {
        let parts: Vec<&str> = [&*input.first_name, &*input.middle_name, &*input.last_name]
            .iter()
            .filter(|s| !s.is_empty())
            .copied()
            .collect();
        if parts.is_empty() {
            input.display_name.clone()
        } else {
            parts.join(" ")
        }
    } else {
        input.display_name.clone()
    };

    let record = NewUser {
        id: Uuid::new_v4().to_string(),
        email_lower: email.to_lowercase(),
        email,
        password_hash: auth::hash_password(&input.password)?,
        display_name,
        avatar_url: input.avatar_url,
        is_active: true,
        first_name: input.first_name,
        middle_name: input.middle_name,
        last_name: input.last_name,
        username: input.username,
    };

    diesel::insert_into(gateway_users::table)
        .values(&record)
        .execute(conn)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Error::Conflict("that email is already registered".into()),
            other => other.into(),
        })?;
    gateway_users::table
        .find(&record.id)
        .select(User::as_select())
        .first(conn)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Error::NotFound("user after insert"),
            other => other.into(),
        })
}

pub fn update(conn: &mut DbConn, id: &str, input: UpdateUser) -> Result<User> {
    if let Some(display_name) = &input.display_name {
        validate::display_name(display_name)?;
    }
    if let Some(avatar_url) = &input.avatar_url {
        validate::avatar_url(avatar_url)?;
    }
    let patch = UserPatch {
        display_name: input.display_name,
        avatar_url: input.avatar_url,
        is_active: input.is_active,
        first_name: input.first_name,
        middle_name: input.middle_name,
        last_name: input.last_name,
        username: input.username,
        updated_at: Utc::now().naive_utc(),
    };
    let n = diesel::update(gateway_users::table.find(id))
        .set(&patch)
        .execute(conn)?;
    if n == 0 {
        return Err(Error::NotFound("user"));
    }
    gateway_users::table
        .find(id)
        .select(User::as_select())
        .first(conn)
        .map_err(|_| Error::NotFound("user after update"))
}

pub fn update_password(conn: &mut DbConn, id: &str, new_hash: &str) -> Result<()> {
    let n = diesel::update(gateway_users::table.find(id))
        .set((
            gateway_users::password_hash.eq(new_hash),
            gateway_users::token_version.eq(gateway_users::token_version + 1),
            gateway_users::updated_at.eq(Utc::now().naive_utc()),
        ))
        .execute(conn)?;
    if n == 0 {
        return Err(Error::NotFound("user"));
    }
    Ok(())
}

pub fn delete(conn: &mut DbConn, id: &str) -> Result<()> {
    let removed = diesel::delete(gateway_users::table.find(id)).execute(conn)?;
    if removed == 0 {
        return Err(Error::NotFound("user"));
    }
    Ok(())
}

pub fn assign_role(conn: &mut DbConn, user_id: &str, role_id: i32) -> Result<()> {
    ensure_user_and_role(conn, user_id, role_id)?;
    diesel::insert_into(gateway_user_roles::table)
        .values((
            gateway_user_roles::user_id.eq(user_id),
            gateway_user_roles::role_id.eq(role_id),
        ))
        .on_conflict_do_nothing()
        .execute(conn)?;
    Ok(())
}

pub fn revoke_role(conn: &mut DbConn, user_id: &str, role_id: i32) -> Result<()> {
    ensure_user_and_role(conn, user_id, role_id)?;
    diesel::delete(
        gateway_user_roles::table
            .filter(gateway_user_roles::user_id.eq(user_id))
            .filter(gateway_user_roles::role_id.eq(role_id)),
    )
    .execute(conn)?;
    Ok(())
}

/// A foreign key that fails at insert time surfaces as a bare 500; checking
/// both sides first turns an unknown id into an honest 404.
fn ensure_user_and_role(conn: &mut DbConn, user_id: &str, role_id: i32) -> Result<()> {
    gateway_users::table
        .find(user_id)
        .select(gateway_users::id)
        .first::<String>(conn)
        .map_err(|_| Error::NotFound("user"))?;
    gateway_roles::table
        .find(role_id)
        .select(gateway_roles::id)
        .first::<i32>(conn)
        .map_err(|_| Error::NotFound("role"))?;
    Ok(())
}

pub fn role_names(conn: &mut DbConn, user_id: &str) -> Result<Vec<String>> {
    roles_on(conn, user_id)
}

fn roles_on(c: &mut DbConn, user_id: &str) -> Result<Vec<String>> {
    Ok(gateway_user_roles::table
        .inner_join(gateway_roles::table)
        .filter(gateway_user_roles::user_id.eq(user_id))
        .select(gateway_roles::name)
        .order(gateway_roles::name.asc())
        .load(c)?)
}

pub fn permission_keys(conn: &mut DbConn, user_id: &str) -> Result<Vec<String>> {
    permissions_on(conn, user_id)
}

fn permissions_on(c: &mut DbConn, user_id: &str) -> Result<Vec<String>> {
    let rows: Vec<(String, String)> = gateway_user_roles::table
        .inner_join(
            gateway_role_permissions::table
                .on(gateway_role_permissions::role_id.eq(gateway_user_roles::role_id)),
        )
        .inner_join(
            gateway_permissions::table
                .on(gateway_permissions::id.eq(gateway_role_permissions::permission_id)),
        )
        .filter(gateway_user_roles::user_id.eq(user_id))
        .select((gateway_permissions::module, gateway_permissions::action))
        .load(c)?;

    let mut keys: Vec<String> = rows.into_iter().map(|(m, a)| format!("{m}:{a}")).collect();
    keys.sort_unstable();
    keys.dedup();
    Ok(keys)
}

pub fn token_version(conn: &mut DbConn, user_id: &str) -> Result<i32> {
    gateway_users::table
        .find(user_id)
        .select(gateway_users::token_version)
        .first(conn)
        .map_err(|_| Error::NotFound("user"))
}

pub fn search(conn: &mut DbConn, q: &str, limit: usize) -> Result<Vec<User>> {
    let pattern = format!("%{q}%");
    let users: Vec<User> = gateway_users::table
        .select(User::as_select())
        .filter(gateway_users::is_active.eq(true))
        .filter(
            gateway_users::email_lower
                .like(&pattern)
                .or(gateway_users::display_name.like(&pattern))
                .or(gateway_users::first_name.like(&pattern))
                .or(gateway_users::last_name.like(&pattern))
                .or(gateway_users::username.like(&pattern)),
        )
        .order(gateway_users::display_name.asc())
        .limit(limit as i64)
        .load(conn)?;
    Ok(users)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> DbPool {
        let pool = db::create_pool_from(":memory:").expect("pool");
        db::run_migrations(&pool).expect("migrations");
        pool
    }

    fn seeded_ids() -> (db::DbPool, String, i32, i32) {
        let pool = pool();
        let user = create(
            &mut pool.get().unwrap(),
            models::CreateUser {
                email: "single-role@example.com".into(),
                password: "password123".into(),
                display_name: "Single Role".into(),
                avatar_url: String::new(),
                first_name: String::new(),
                middle_name: String::new(),
                last_name: String::new(),
                username: String::new(),
            },
        )
        .expect("user");
        let mut conn = pool.get().unwrap();
        let roles = crate::roles::list(&conn).expect("roles");
        let admin = roles
            .iter()
            .find(|r| r.role.name == "admin")
            .unwrap()
            .role
            .id;
        let viewer = roles
            .iter()
            .find(|r| r.role.name == "viewer")
            .unwrap()
            .role
            .id;
        (pool, user.id, admin, viewer)
    }

    #[test]
    fn assigning_a_second_role_does_not_replace_the_first() {
        let (pool, uid, admin, viewer) = seeded_ids();
        assign_role(&mut pool.get().unwrap(), &uid, admin).expect("first grant");
        assign_role(&mut pool.get().unwrap(), &uid, viewer).expect("second grant");
        let roles = role_names(&mut pool.get().unwrap(), &uid).unwrap();
        assert_eq!(roles, vec!["admin", "viewer"], "user can have multiple roles");
    }

    #[test]
    fn revoking_leaves_the_user_without_roles_but_assignable_again() {
        let (pool, uid, admin, viewer) = seeded_ids();
        assign_role(&mut pool.get().unwrap(), &uid, admin).expect("grant");
        revoke_role(&mut pool.get().unwrap(), &uid, admin).expect("revoke");
        assert!(role_names(&mut pool.get().unwrap(), &uid).unwrap().is_empty());
        assign_role(&mut pool.get().unwrap(), &uid, viewer).expect("re-grant");
        assert_eq!(role_names(&mut pool.get().unwrap(), &uid).unwrap(), vec!["viewer"]);
    }
}
