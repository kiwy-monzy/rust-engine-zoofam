use db::DbPool;
use models::{CreateUser, UpdateUser, User, UserWithRoles};

pub fn list(pool: &DbPool) -> Result<Vec<UserWithRoles>, controller::Error> {
    controller::users::list(pool)
}

pub fn get(pool: &DbPool, id: &str) -> Result<UserWithRoles, controller::Error> {
    controller::users::get(pool, id)
}

pub fn get_by_email(pool: &DbPool, email: &str) -> Result<User, controller::Error> {
    controller::users::find_by_email(pool, email)
}

pub fn search(pool: &DbPool, q: &str, limit: usize) -> Result<Vec<User>, controller::Error> {
    controller::users::search(pool, q, limit)
}

pub fn create(pool: &DbPool, input: CreateUser) -> Result<User, controller::Error> {
    controller::users::create(pool, input)
}

pub fn update(pool: &DbPool, id: &str, input: UpdateUser) -> Result<User, controller::Error> {
    controller::users::update(pool, id, input)
}

pub fn delete(pool: &DbPool, id: &str) -> Result<(), controller::Error> {
    controller::users::delete(pool, id)
}

pub fn assign_role(pool: &DbPool, user_id: &str, role_id: i32) -> Result<(), controller::Error> {
    controller::users::assign_role(pool, user_id, role_id)
}

pub fn revoke_role(pool: &DbPool, user_id: &str, role_id: i32) -> Result<(), controller::Error> {
    controller::users::revoke_role(pool, user_id, role_id)
}

pub fn role_names(pool: &DbPool, user_id: &str) -> Result<Vec<String>, controller::Error> {
    controller::users::role_names(pool, user_id)
}

pub fn permission_keys(pool: &DbPool, user_id: &str) -> Result<Vec<String>, controller::Error> {
    controller::users::permission_keys(pool, user_id)
}

pub fn update_password(
    pool: &DbPool,
    user_id: &str,
    new_hash: &str,
) -> Result<(), controller::Error> {
    controller::users::update_password(pool, user_id, new_hash)
}
