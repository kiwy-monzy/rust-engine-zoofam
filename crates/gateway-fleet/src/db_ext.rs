use chrono::{DateTime, NaiveDateTime, Utc};
use db::{DbError, DbPool};
use diesel::prelude::*;

use crate::VaultEntry;
use models::schema::gateway_vault;

fn now_naive() -> NaiveDateTime {
    Utc::now().naive_utc()
}

// ---------------------------------------------------------------- vault --

#[derive(Queryable, Selectable)]
#[diesel(table_name = gateway_vault)]
struct VaultRow {
    id: String,
    #[allow(dead_code)]
    user_id: String,
    service: String,
    kind: String,
    name: String,
    secret: String,
    meta: String,
    is_active: bool,
    #[allow(dead_code)]
    created_at: NaiveDateTime,
    #[allow(dead_code)]
    updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = gateway_vault)]
struct NewVault<'a> {
    service: &'a str,
    kind: &'a str,
    name: &'a str,
    secret: &'a str,
    meta: &'a str,
    is_active: bool,
}

pub fn vault_put(
    pool: &DbPool,
    service: &str,
    kind: &str,
    name: &str,
    secret: &str,
    meta: &serde_json::Value,
) -> Result<(), DbError> {
    let mut conn = db::conn(pool)?;
    let meta_str = meta.to_string();
    let updated = diesel::update(
        gateway_vault::table
            .filter(gateway_vault::service.eq(service))
            .filter(gateway_vault::name.eq(name)),
    )
    .set((
        gateway_vault::secret.eq(secret),
        gateway_vault::kind.eq(kind),
        gateway_vault::meta.eq(&meta_str),
        gateway_vault::is_active.eq(true),
        gateway_vault::updated_at.eq(now_naive()),
    ))
    .execute(&mut conn)?;
    if updated == 0 {
        diesel::insert_into(gateway_vault::table)
            .values(NewVault {
                service,
                kind,
                name,
                secret,
                meta: &meta_str,
                is_active: true,
            })
            .execute(&mut conn)?;
    }
    Ok(())
}

pub fn vault_list(pool: &DbPool) -> Result<Vec<VaultEntry>, DbError> {
    let mut conn = db::conn(pool)?;
    let rows: Vec<VaultRow> = gateway_vault::table
        .order((gateway_vault::service.asc(), gateway_vault::name.asc()))
        .load(&mut conn)?;
    Ok(rows
        .into_iter()
        .map(|r| VaultEntry {
            id: r.id.parse().unwrap_or_default(),
            service: r.service,
            kind: r.kind,
            name: r.name,
            meta: serde_json::from_str(&r.meta).unwrap_or(serde_json::json!({})),
            is_active: r.is_active,
            has_secret: !r.secret.is_empty(),
            updated_at: DateTime::<Utc>::from_naive_utc_and_offset(r.updated_at, Utc).to_rfc3339(),
        })
        .collect())
}

pub fn vault_secret(pool: &DbPool, service: &str, name: &str) -> Result<Option<String>, DbError> {
    let mut conn = db::conn(pool)?;
    let row: Option<VaultRow> = gateway_vault::table
        .filter(gateway_vault::service.eq(service))
        .filter(gateway_vault::name.eq(name))
        .filter(gateway_vault::is_active.eq(true))
        .first(&mut conn)
        .optional()?;
    Ok(row.map(|r| r.secret).filter(|s| !s.is_empty()))
}

pub fn vault_get(
    pool: &DbPool,
    service: &str,
    name: &str,
) -> Result<Option<(String, serde_json::Value)>, DbError> {
    let mut conn = db::conn(pool)?;
    let row: Option<VaultRow> = gateway_vault::table
        .filter(gateway_vault::service.eq(service))
        .filter(gateway_vault::name.eq(name))
        .filter(gateway_vault::is_active.eq(true))
        .first(&mut conn)
        .optional()?;
    Ok(row.map(|r| {
        let meta = serde_json::from_str(&r.meta).unwrap_or(serde_json::json!({}));
        (r.secret, meta)
    }))
}
