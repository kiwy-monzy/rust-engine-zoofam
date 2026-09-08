use diesel::prelude::*;

use db::{conn, DbPool};
use models::schema::gateway_system;
use models::SystemSettingRow;

use crate::{Error, Result};

pub const DEFAULT_NAME: &str = "MkulimaLink";
pub const DEFAULT_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Default)]
pub struct SystemSettings {
    pub app_name: String,
    pub maintenance_mode: String,
    pub registration_enabled: String,
    pub version: String,
}

impl SystemSettings {
    pub fn from_rows(rows: Vec<SystemSettingRow>) -> Self {
        let mut settings = Self::default();
        for row in rows {
            match row.key.as_str() {
                "app_name" => settings.app_name = row.value,
                "maintenance_mode" => settings.maintenance_mode = row.value,
                "registration_enabled" => settings.registration_enabled = row.value,
                "version" => settings.version = row.value,
                _ => {}
            }
        }
        if settings.app_name.is_empty() {
            settings.app_name = DEFAULT_NAME.to_string();
        }
        if settings.version.is_empty() {
            settings.version = DEFAULT_VERSION.to_string();
        }
        settings
    }
}

pub fn get(pool: &DbPool) -> Result<SystemSettings> {
    let mut c = conn(pool)?;
    let rows: Vec<SystemSettingRow> = gateway_system::table
        .select(SystemSettingRow::as_select())
        .load(&mut c)
        .map_err(|_| Error::NotFound("system"))?;
    Ok(SystemSettings::from_rows(rows))
}

pub fn update(pool: &DbPool, key: &str, value: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let now = chrono::Utc::now().naive_utc();

    diesel::update(gateway_system::table.filter(gateway_system::key.eq(key)))
        .set((
            gateway_system::value.eq(value),
            gateway_system::updated_at.eq(now),
        ))
        .execute(&mut c)
        .map_err(|e| {
            if e == diesel::result::Error::NotFound {
                Error::NotFound("system setting")
            } else {
                Error::Db(db::DbError::Query(e))
            }
        })?;
    Ok(())
}

pub fn get_value(pool: &DbPool, key: &str) -> Result<Option<String>> {
    let mut c = conn(pool)?;
    let result: Option<SystemSettingRow> = gateway_system::table
        .filter(gateway_system::key.eq(key))
        .select(SystemSettingRow::as_select())
        .first(&mut c)
        .ok();
    Ok(result.map(|r| r.value))
}

pub fn set_value(pool: &DbPool, key: &str, value: &str) -> Result<()> {
    let mut c = conn(pool)?;
    let now = chrono::Utc::now().naive_utc();

    diesel::insert_into(gateway_system::table)
        .values((
            gateway_system::key.eq(key),
            gateway_system::value.eq(value),
            gateway_system::updated_at.eq(now),
        ))
        .on_conflict(gateway_system::key)
        .do_update()
        .set((
            gateway_system::value.eq(value),
            gateway_system::updated_at.eq(now),
        ))
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
    fn system_settings_are_readable() {
        let pool = pool();
        let s = get(&pool).expect("seeded rows");
        assert_eq!(s.app_name, "MkulimaLink");
        assert!(!s.version.is_empty());
    }

    #[test]
    fn system_values_can_be_set_and_get() {
        let pool = pool();
        set_value(&pool, "app_name", "Test App").expect("set value");
        let value = get_value(&pool, "app_name").expect("get value");
        assert_eq!(value, Some("Test App".to_string()));
    }
}
