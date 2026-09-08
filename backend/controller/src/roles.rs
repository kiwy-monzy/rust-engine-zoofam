use diesel::prelude::*;

use db::{conn, DbConn, DbPool};
use models::schema::{gateway_permissions, gateway_role_permissions, gateway_roles};
use models::{CreateRole, NewRole, Permission, Role, RoleWithPermissions};

use crate::{Error, Result};
use models::UpdateRole;

pub fn list(conn: &mut DbConn) -> Result<Vec<RoleWithPermissions>> {
    let roles: Vec<Role> = gateway_roles::table
        .select(Role::as_select())
        .order(gateway_roles::name.asc())
        .load(conn)?;

    let mut out = Vec::with_capacity(roles.len());
    for role in roles {
        let permissions = permissions_on(conn, role.id)?;
        out.push(RoleWithPermissions { role, permissions });
    }
    Ok(out)
}

pub fn get(conn: &mut DbConn, id: i32) -> Result<RoleWithPermissions> {
    let role: Role = gateway_roles::table
        .find(id)
        .select(Role::as_select())
        .first(conn)
        .map_err(|_| Error::NotFound("role"))?;
    let permissions = permissions_on(conn, role.id)?;
    Ok(RoleWithPermissions { role, permissions })
}

pub fn create(pool: &DbPool, input: CreateRole) -> Result<Role> {
    let name = input.name.trim().to_lowercase();
    if name.is_empty() {
        return Err(Error::Invalid("a role needs a name".into()));
    }
    let record = NewRole {
        name,
        description: input.description,
    };

    let mut c = conn(pool)?;
    diesel::insert_into(gateway_roles::table)
        .values(&record)
        .execute(&mut c)
        .map_err(|e| match e {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => Error::Conflict("that role already exists".into()),
            other => other.into(),
        })?;
    gateway_roles::table
        .filter(gateway_roles::name.eq(&record.name))
        .select(Role::as_select())
        .first(&mut c)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Error::NotFound("role after insert"),
            other => other.into(),
        })
}

pub fn update(pool: &DbPool, id: i32, input: UpdateRole) -> Result<Role> {
    let name = input
        .name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty());
    if input.name.is_some() && name.is_none() {
        return Err(Error::Invalid("a role needs a name".into()));
    }
    if name.is_none() && input.description.is_none() {
        return Err(Error::Invalid("nothing to update".into()));
    }
    let patch = models::UpdateRole {
        name: name.map(str::to_lowercase),
        description: input.description,
    };

    let mut c = conn(pool)?;
    let n = diesel::update(gateway_roles::table.find(id))
        .set(&patch)
        .execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("role"));
    }
    gateway_roles::table
        .find(id)
        .select(Role::as_select())
        .first(&mut c)
        .map_err(|_| Error::NotFound("role after update"))
}

pub fn delete(pool: &DbPool, id: i32) -> Result<()> {
    let mut c = conn(pool)?;
    let removed = diesel::delete(gateway_roles::table.find(id)).execute(&mut c)?;
    if removed == 0 {
        return Err(Error::NotFound("role"));
    }
    Ok(())
}

/// A foreign key that fails at insert time surfaces as a bare 500; checking
/// both sides first turns an unknown id into an honest 404.
fn ensure_role_and_permission(c: &mut DbConn, role_id: i32, permission_id: i32) -> Result<()> {
    gateway_roles::table
        .find(role_id)
        .select(gateway_roles::id)
        .first::<i32>(c)
        .map_err(|_| Error::NotFound("role"))?;
    gateway_permissions::table
        .find(permission_id)
        .select(gateway_permissions::id)
        .first::<i32>(c)
        .map_err(|_| Error::NotFound("permission"))?;
    Ok(())
}

pub fn grant(pool: &DbPool, role_id: i32, permission_id: i32) -> Result<()> {
    let mut c = conn(pool)?;
    ensure_role_and_permission(&mut c, role_id, permission_id)?;
    diesel::insert_into(gateway_role_permissions::table)
        .values((
            gateway_role_permissions::role_id.eq(role_id),
            gateway_role_permissions::permission_id.eq(permission_id),
        ))
        .on_conflict_do_nothing()
        .execute(&mut c)?;
    Ok(())
}

pub fn revoke(pool: &DbPool, role_id: i32, permission_id: i32) -> Result<()> {
    let mut c = conn(pool)?;
    ensure_role_and_permission(&mut c, role_id, permission_id)?;
    diesel::delete(
        gateway_role_permissions::table
            .filter(gateway_role_permissions::role_id.eq(role_id))
            .filter(gateway_role_permissions::permission_id.eq(permission_id)),
    )
    .execute(&mut c)?;
    Ok(())
}

pub fn permissions_of(pool: &DbPool, role_id: i32) -> Result<Vec<Permission>> {
    let mut c = conn(pool)?;
    permissions_on(&mut c, role_id)
}

fn permissions_on(c: &mut DbConn, role_id: i32) -> Result<Vec<Permission>> {
    Ok(gateway_role_permissions::table
        .inner_join(gateway_permissions::table)
        .filter(gateway_role_permissions::role_id.eq(role_id))
        .select(Permission::as_select())
        .order((
            gateway_permissions::module.asc(),
            gateway_permissions::action.asc(),
        ))
        .load(c)?)
}
