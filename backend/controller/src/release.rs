use diesel::prelude::*;

use db::{conn, DbPool};
use models::schema::gateway_releases;
use models::{NewRelease, Release};

use crate::{Error, Result};

pub fn list(pool: &DbPool) -> Result<Vec<Release>> {
    let mut c = conn(pool)?;
    gateway_releases::table
        .order(gateway_releases::created_at.desc())
        .select(Release::as_select())
        .load(&mut c)
        .map_err(Error::from)
}

pub fn list_by_platform(pool: &DbPool, platform: &str) -> Result<Vec<Release>> {
    let mut c = conn(pool)?;
    gateway_releases::table
        .filter(gateway_releases::platform.eq(platform))
        .order(gateway_releases::created_at.desc())
        .select(Release::as_select())
        .load(&mut c)
        .map_err(Error::from)
}

pub fn latest(pool: &DbPool, platform: &str) -> Result<Option<Release>> {
    let mut c = conn(pool)?;
    gateway_releases::table
        .filter(gateway_releases::platform.eq(platform))
        .order(gateway_releases::created_at.desc())
        .select(Release::as_select())
        .first(&mut c)
        .optional()
        .map_err(Error::from)
}

pub fn latest_any(pool: &DbPool) -> Result<Option<Release>> {
    let mut c = conn(pool)?;
    gateway_releases::table
        .order(gateway_releases::created_at.desc())
        .select(Release::as_select())
        .first(&mut c)
        .optional()
        .map_err(Error::from)
}

pub fn get(pool: &DbPool, id: i32) -> Result<Release> {
    let mut c = conn(pool)?;
    gateway_releases::table
        .find(id)
        .select(Release::as_select())
        .first(&mut c)
        .map_err(|_| Error::NotFound("release"))
}

pub fn create(pool: &DbPool, input: NewRelease) -> Result<Release> {
    let mut c = conn(pool)?;
    diesel::insert_into(gateway_releases::table)
        .values(&input)
        .execute(&mut c)
        .map_err(Error::from)?;
    gateway_releases::table
        .filter(gateway_releases::version.eq(&input.version))
        .filter(gateway_releases::platform.eq(&input.platform))
        .select(Release::as_select())
        .first(&mut c)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => Error::NotFound("release after insert"),
            other => Error::from(other),
        })
}

pub fn increment_download(pool: &DbPool, id: i32) -> Result<Release> {
    let mut c = conn(pool)?;
    let n = diesel::update(gateway_releases::table.find(id))
        .set(gateway_releases::download_count.eq(gateway_releases::download_count + 1))
        .execute(&mut c)?;
    if n == 0 {
        return Err(Error::NotFound("release"));
    }
    gateway_releases::table
        .find(id)
        .select(Release::as_select())
        .first(&mut c)
        .map_err(|_| Error::NotFound("release after update"))
}

pub fn delete(pool: &DbPool, id: i32) -> Result<()> {
    let mut c = conn(pool)?;
    let deleted = diesel::delete(gateway_releases::table.find(id)).execute(&mut c)?;
    if deleted == 0 {
        return Err(Error::NotFound("release"));
    }
    Ok(())
}

pub fn find_by_version_platform(
    pool: &DbPool,
    version: &str,
    platform: &str,
) -> Result<Option<Release>> {
    let mut c = conn(pool)?;
    gateway_releases::table
        .filter(gateway_releases::version.eq(version))
        .filter(gateway_releases::platform.eq(platform))
        .select(Release::as_select())
        .first(&mut c)
        .optional()
        .map_err(Error::from)
}

pub fn find_latest_for_platform(pool: &DbPool, platform: &str) -> Result<Option<Release>> {
    let mut c = conn(pool)?;
    gateway_releases::table
        .filter(gateway_releases::platform.eq(platform))
        .order(gateway_releases::created_at.desc())
        .select(Release::as_select())
        .first(&mut c)
        .optional()
        .map_err(Error::from)
}
