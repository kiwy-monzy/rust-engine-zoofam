use diesel::prelude::*;

use db::{conn, DbPool};
use models::schema::gateway_permissions;
use models::{NewPermission, Permission, UpdatePermission};

use crate::{Error, Result};

pub fn list(pool: &DbPool) -> Result<Vec<Permission>> {
    let mut c = conn(pool)?;
    Ok(gateway_permissions::table
        .select(Permission::as_select())
        .order((
            gateway_permissions::module.asc(),
            gateway_permissions::action.asc(),
        ))
        .load(&mut c)?)
}

pub fn get(pool: &DbPool, id: i32) -> Result<Permission> {
    let mut c = conn(pool)?;
    gateway_permissions::table
        .find(id)
        .select(Permission::as_select())
        .first(&mut c)
        .map_err(|_| Error::NotFound("permission"))
}

pub fn create(pool: &DbPool, input: NewPermission) -> Result<Permission> {
    let mut c = conn(pool)?;
    diesel::insert_into(gateway_permissions::table)
        .values(&input)
        .execute(&mut c)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Error::Conflict("that permission already exists".into()),
            other => other.into(),
        })?;
    gateway_permissions::table
        .filter(gateway_permissions::module.eq(&input.module))
        .filter(gateway_permissions::action.eq(&input.action))
        .select(Permission::as_select())
        .first(&mut c)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Error::NotFound("permission after insert"),
            other => other.into(),
        })
}

pub fn update(pool: &DbPool, id: i32, input: UpdatePermission) -> Result<Permission> {
    let module = input
        .module
        .as_deref()
        .map(str::trim)
        .filter(|m| !m.is_empty());
    let action = input
        .action
        .as_deref()
        .map(str::trim)
        .filter(|a| !a.is_empty());
    let description = input.description.as_deref().map(str::trim);
    if input.module.is_some() && module.is_none() {
        return Err(Error::Invalid("a permission needs a module".into()));
    }
    if input.action.is_some() && action.is_none() {
        return Err(Error::Invalid("a permission needs an action".into()));
    }
    if module.is_none() && action.is_none() && description.is_none() {
        return Err(Error::Invalid("nothing to update".into()));
    }
    let patch = UpdatePermission {
        module: module.map(str::to_lowercase),
        action: action.map(str::to_lowercase),
        description: description.map(str::to_string),
    };

    let mut c = conn(pool)?;
    let n = diesel::update(gateway_permissions::table.find(id))
        .set(&patch)
        .execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("permission"));
    }
    gateway_permissions::table
        .find(id)
        .select(Permission::as_select())
        .first(&mut c)
        .map_err(|_| Error::NotFound("permission after update"))
}

pub fn delete(pool: &DbPool, id: i32) -> Result<()> {
    let mut c = conn(pool)?;
    let removed = diesel::delete(gateway_permissions::table.find(id)).execute(&mut c)?;
    if removed == 0 {
        return Err(Error::NotFound("permission"));
    }
    Ok(())
}
