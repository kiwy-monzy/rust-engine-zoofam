use db::DbPool;
use models::{NewPermission, Permission, UpdatePermission};

pub fn list(pool: &DbPool) -> Result<Vec<Permission>, controller::Error> {
    controller::permissions::list(pool)
}

pub fn create(pool: &DbPool, input: NewPermission) -> Result<Permission, controller::Error> {
    controller::permissions::create(pool, input)
}

pub fn update(
    pool: &DbPool,
    id: i32,
    input: UpdatePermission,
) -> Result<Permission, controller::Error> {
    controller::permissions::update(pool, id, input)
}

pub fn delete(pool: &DbPool, id: i32) -> Result<(), controller::Error> {
    controller::permissions::delete(pool, id)
}
