use db::DbPool;
use models::{CreateRole, Role, RoleWithPermissions, UpdateRole};

pub fn list(pool: &DbPool) -> Result<Vec<RoleWithPermissions>, controller::Error> {
    controller::roles::list(pool)
}

pub fn get(pool: &DbPool, id: i32) -> Result<RoleWithPermissions, controller::Error> {
    controller::roles::get(pool, id)
}

pub fn create(pool: &DbPool, input: CreateRole) -> Result<Role, controller::Error> {
    controller::roles::create(pool, input)
}

pub fn update(pool: &DbPool, id: i32, input: UpdateRole) -> Result<Role, controller::Error> {
    controller::roles::update(pool, id, input)
}

pub fn delete(pool: &DbPool, id: i32) -> Result<(), controller::Error> {
    controller::roles::delete(pool, id)
}

pub fn grant(pool: &DbPool, role_id: i32, permission_id: i32) -> Result<(), controller::Error> {
    controller::roles::grant(pool, role_id, permission_id)
}

pub fn revoke(pool: &DbPool, role_id: i32, permission_id: i32) -> Result<(), controller::Error> {
    controller::roles::revoke(pool, role_id, permission_id)
}
