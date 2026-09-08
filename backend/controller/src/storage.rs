use crate::{Controllers, Error, Result};
use chrono::Utc;
use diesel::prelude::*;
use models::schema::user_files;
use models::{NewUserFile, UserFile};
use std::fs;
use std::path::PathBuf;

const STORAGE_ROOT: &str = "storage";

fn storage_root() -> PathBuf {
    PathBuf::from(STORAGE_ROOT)
}

fn user_dir(user_id: &str) -> PathBuf {
    storage_root().join(user_id)
}

fn collection_dir(user_id: &str, collection: &str) -> PathBuf {
    user_dir(user_id).join(collection)
}

fn file_path(user_id: &str, collection: &str, filename: &str) -> PathBuf {
    collection_dir(user_id, collection).join(filename)
}

fn safe_collection(s: &str) -> Result<String> {
    let c = s.trim().to_lowercase();
    if c.is_empty() || c.len() > 64 || c.contains("..") || c.contains('/') || c.contains('\\') {
        return Err(Error::Invalid("invalid collection name".into()));
    }
    Ok(c)
}

fn safe_filename(s: &str) -> Result<String> {
    let f = s.trim().to_string();
    if f.is_empty() || f.len() > 255 || f.contains("..") || f.contains('/') || f.contains('\\') {
        return Err(Error::Invalid("invalid filename".into()));
    }
    Ok(f)
}

impl Controllers {
    /// Persist a file under `user_id/collection/filename` and write the
    /// `user_files` row in one call. Used by avatar uploads, GeoJSON
    /// imports, etc. Returns the saved `UserFile` (with its assigned id).
    pub fn store_user_file(
        &self,
        user_id: &str,
        collection: &str,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> Result<UserFile> {
        self.upload_file(user_id, collection, filename, mime_type, data)
    }

    pub fn upload_file(
        &self,
        user_id: &str,
        collection: &str,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> Result<UserFile> {
        let collection = safe_collection(collection)?;
        let filename = safe_filename(filename)?;

        let dir = collection_dir(user_id, &collection);
        fs::create_dir_all(&dir)
            .map_err(|e| Error::Invalid(format!("failed to create storage dir: {e}")))?;

        let path = file_path(user_id, &collection, &filename);
        fs::write(&path, data).map_err(|e| Error::Invalid(format!("failed to write file: {e}")))?;

        let now = Utc::now().naive_utc();
        let storage_path = path.to_string_lossy().into_owned();
        let new_file = NewUserFile {
            user_id: user_id.to_string(),
            collection: collection.to_string(),
            filename: filename.to_string(),
            original_name: filename.to_string(),
            mime_type: mime_type.to_string(),
            size_bytes: data.len() as i32,
            storage_path,
            is_public: false,
            created_at: now,
            updated_at: now,
        };

        let mut conn = self.pool.get().map_err(|e| Error::Db(e.into()))?;

        // Upsert: if file exists, update size and mime
        let existing: Option<UserFile> = user_files::table
            .filter(user_files::user_id.eq(user_id))
            .filter(user_files::collection.eq(&new_file.collection))
            .filter(user_files::filename.eq(&new_file.filename))
            .select(UserFile::as_select())
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Db(e.into()))?;

        if let Some(_old) = existing {
            diesel::update(user_files::table)
                .filter(user_files::user_id.eq(user_id))
                .filter(user_files::collection.eq(&new_file.collection))
                .filter(user_files::filename.eq(&new_file.filename))
                .set((
                    user_files::mime_type.eq(&new_file.mime_type),
                    user_files::size_bytes.eq(new_file.size_bytes),
                    user_files::updated_at.eq(now),
                ))
                .execute(&mut conn)
                .map_err(|e| Error::Db(e.into()))?;
        } else {
            diesel::insert_into(user_files::table)
                .values(&new_file)
                .execute(&mut conn)
                .map_err(|e| Error::Db(e.into()))?;
        }

        self.get_file(user_id, &new_file.collection, &new_file.filename)
    }

    pub fn get_file(&self, user_id: &str, collection: &str, filename: &str) -> Result<UserFile> {
        let mut conn = self.pool.get().map_err(|e| Error::Db(e.into()))?;
        user_files::table
            .filter(user_files::user_id.eq(user_id))
            .filter(user_files::collection.eq(collection))
            .filter(user_files::filename.eq(filename))
            .select(UserFile::as_select())
            .first(&mut conn)
            .map_err(|_| Error::NotFound("file"))
    }

    pub fn list_files(&self, user_id: &str, collection: Option<&str>) -> Result<Vec<UserFile>> {
        let mut conn = self.pool.get().map_err(|e| Error::Db(e.into()))?;
        let mut query = user_files::table
            .filter(user_files::user_id.eq(user_id))
            .select(UserFile::as_select())
            .order(user_files::created_at.desc())
            .into_boxed();
        if let Some(c) = collection {
            query = query.filter(user_files::collection.eq(c));
        }
        query.load(&mut conn).map_err(|e| Error::Db(e.into()))
    }

    pub fn delete_file(&self, user_id: &str, collection: &str, filename: &str) -> Result<()> {
        let mut conn = self.pool.get().map_err(|e| Error::Db(e.into()))?;
        let removed = diesel::delete(
            user_files::table
                .filter(user_files::user_id.eq(user_id))
                .filter(user_files::collection.eq(collection))
                .filter(user_files::filename.eq(filename)),
        )
        .execute(&mut conn)
        .map_err(|e| Error::Db(e.into()))?;

        if removed > 0 {
            let path = file_path(user_id, collection, filename);
            let _ = fs::remove_file(path);
        }
        Ok(())
    }

    pub fn read_file(&self, user_id: &str, collection: &str, filename: &str) -> Result<Vec<u8>> {
        let path = file_path(user_id, collection, filename);
        if !path.exists() {
            return Err(Error::NotFound("file"));
        }
        fs::read(&path).map_err(|e| Error::Invalid(format!("failed to read file: {e}")))
    }

    pub fn file_path_on_disk(
        &self,
        user_id: &str,
        collection: &str,
        filename: &str,
    ) -> Result<PathBuf> {
        let path = file_path(user_id, collection, filename);
        if !path.exists() {
            return Err(Error::NotFound("file"));
        }
        Ok(path)
    }
}
