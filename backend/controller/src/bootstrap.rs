use diesel::prelude::*;
use models::{NewUser, UserRole};
use uuid::Uuid;

use models::schema::{gateway_roles, gateway_user_roles, gateway_users};

use crate::{validate, Error, Result};

pub const DEFAULT_ADMIN_EMAIL: &str = "admin@example.com";
pub const DEFAULT_ADMIN_PASSWORD: &str = "admin12345";
pub const DEFAULT_ADMIN_NAME: &str = "System Admin";

pub fn ensure_admin(pool: &db::DbPool) -> Result<()> {
    let email = std::env::var("ADMIN_EMAIL").unwrap_or_else(|_| DEFAULT_ADMIN_EMAIL.to_string());
    let password =
        std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| DEFAULT_ADMIN_PASSWORD.to_string());
    ensure_admin_with(pool, &email, &password)
}

pub fn ensure_admin_with(pool: &db::DbPool, email: &str, password: &str) -> Result<()> {
    if std::env::var("ADMIN_PASSWORD").is_err() && password == DEFAULT_ADMIN_PASSWORD {
        tracing::warn!(
            "ADMIN_PASSWORD is unset; the bootstrap admin keeps the default password - set ADMIN_PASSWORD"
        );
    }
    let email = validate::email(email)?;
    validate::password(password)?;
    let email_lower = email.to_lowercase();

    let mut c = db::conn(pool)?;

    let user_id = match gateway_users::table
        .filter(gateway_users::email_lower.eq(&email_lower))
        .select(gateway_users::id)
        .first::<String>(&mut c)
    {
        Ok(id) => {
            tracing::info!("bootstrap admin {email} already exists");
            id
        }
        Err(diesel::result::Error::NotFound) => {
            let record = NewUser {
                id: Uuid::new_v4().to_string(),
                email_lower,
                email: email.clone(),
                password_hash: auth::hash_password(password)?,
                display_name: DEFAULT_ADMIN_NAME.to_string(),
                avatar_url: String::new(),
                is_active: true,
                first_name: "System".to_string(),
                middle_name: String::new(),
                last_name: "Admin".to_string(),
                username: "admin".to_string(),
            };
            let id = record.id.clone();
            diesel::insert_into(gateway_users::table)
                .values(&record)
                .execute(&mut c)?;
            tracing::info!("created bootstrap admin {email}");
            id
        }
        Err(e) => return Err(e.into()),
    };

    let admin_role_id: i32 = gateway_roles::table
        .filter(gateway_roles::name.eq("admin"))
        .select(gateway_roles::id)
        .first(&mut c)
        .map_err(|_| Error::NotFound("role"))?;

    diesel::insert_into(gateway_user_roles::table)
        .values(UserRole {
            user_id,
            role_id: admin_role_id,
        })
        .on_conflict_do_nothing()
        .execute(&mut c)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> db::DbPool {
        let pool = db::create_pool_from(":memory:").expect("pool");
        db::run_migrations(&pool).expect("migrations");
        pool
    }

    #[test]
    fn bootstrap_creates_the_admin_once_and_only_once() {
        let pool = pool();
        ensure_admin_with(&pool, "root@gateway.test", "sup3r-secret").expect("first run");

        let mut conn = pool.get().unwrap();
        let user = crate::users::find_by_email(&mut conn, "root@gateway.test").expect("user");
        assert!(user.is_active);
        assert_eq!(
            crate::users::role_names(&mut conn, &user.id).expect("roles"),
            vec!["admin"]
        );

        auth::verify_password("sup3r-secret", &user.password_hash).expect("password matches");

        assert!(crate::users::permission_keys(&mut conn, &user.id)
            .expect("perms")
            .contains(&"users:write".to_string()));

        ensure_admin_with(&pool, "root@gateway.test", "sup3r-secret").expect("idempotent rerun");
        let mut conn2 = pool.get().unwrap();
        assert_eq!(
            crate::users::role_names(&mut conn2, &user.id).expect("roles still single"),
            vec!["admin"]
        );
    }

    #[test]
    fn a_registered_email_is_adopted_not_duplicated() {
        let pool = pool();
        let mut conn = pool.get().unwrap();
        crate::users::create(
            &mut conn,
            models::CreateUser {
                email: "claimed@example.com".into(),
                password: "password123".into(),
                display_name: "Claimed".into(),
                avatar_url: String::new(),
                first_name: String::new(),
                middle_name: String::new(),
                last_name: String::new(),
                username: String::new(),
            },
        )
        .expect("register");

        let mut conn2 = pool.get().unwrap();
        ensure_admin_with(&pool, "CLAIMED@example.com", "another-pass-123").expect("adopt");

        let found = crate::users::find_by_email(&mut conn2, "claimed@example.com").expect("one user");
        assert_eq!(
            crate::users::role_names(&mut conn2, &found.id).expect("granted"),
            vec!["admin"]
        );
    }

    #[test]
    fn weak_bootstrap_credentials_are_refused() {
        let pool = pool();
        let err = ensure_admin_with(&pool, "x@example.com", "short").unwrap_err();
        assert!(matches!(err, Error::Invalid(_)));
    }
}
