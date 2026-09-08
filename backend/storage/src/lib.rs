use chrono::Utc;
use diesel::prelude::*;
use models::schema::user_files;
use models::{NewUserFile, UserFile};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("not found: {0}")]
    NotFound(&'static str),
    #[error("database: {0}")]
    Db(#[from] diesel::result::Error),
    #[error("pool: {0}")]
    Pool(#[from] r2d2::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct StorageService {
    root: PathBuf,
}

impl StorageService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn from_env() -> Self {
        let root = std::env::var("STORAGE_DIR").unwrap_or_else(|_| "storage".into());
        Self::new(root)
    }
    fn collection_dir(&self, uid: &str, c: &str) -> PathBuf {
        self.root.join(uid).join(c)
    }
    fn file_path(&self, uid: &str, c: &str, f: &str) -> PathBuf {
        self.collection_dir(uid, c).join(f)
    }

    pub fn upload_file(
        &self,
        conn: &mut PgConn,
        user_id: &str,
        collection: &str,
        filename: &str,
        mime_type: &str,
        data: &[u8],
    ) -> Result<UserFile> {
        let collection = safe_col(collection)?;
        let filename = safe_fn(filename)?;
        fs::create_dir_all(self.collection_dir(user_id, &collection))?;
        fs::write(self.file_path(user_id, &collection, &filename), data)?;
        let now = Utc::now().naive_utc();
        let storage_path = format!("{}/{}/{}", user_id, collection, filename);
        let new = NewUserFile {
            user_id: user_id.to_string(),
            collection,
            filename: filename.to_string(),
            original_name: filename.to_string(),
            mime_type: mime_type.to_string(),
            size_bytes: data.len() as i32,
            storage_path,
            is_public: false,
            created_at: now,
            updated_at: now,
        };
        let existing: Option<UserFile> = user_files::table
            .filter(user_files::user_id.eq(user_id))
            .filter(user_files::collection.eq(&new.collection))
            .filter(user_files::filename.eq(&new.filename))
            .select(UserFile::as_select())
            .first(conn)
            .optional()?;
        if existing.is_some() {
            diesel::update(user_files::table)
                .filter(user_files::user_id.eq(user_id))
                .filter(user_files::collection.eq(&new.collection))
                .filter(user_files::filename.eq(&new.filename))
                .set((
                    user_files::mime_type.eq(&new.mime_type),
                    user_files::size_bytes.eq(new.size_bytes),
                ))
                .execute(conn)?;
        } else {
            diesel::insert_into(user_files::table)
                .values(&new)
                .execute(conn)?;
        }
        self.get_file(conn, user_id, &new.collection, &new.filename)
    }
    pub fn get_file(&self, conn: &mut PgConn, uid: &str, c: &str, f: &str) -> Result<UserFile> {
        user_files::table
            .filter(user_files::user_id.eq(uid))
            .filter(user_files::collection.eq(c))
            .filter(user_files::filename.eq(f))
            .select(UserFile::as_select())
            .first(conn)
            .map_err(|_| Error::NotFound("file"))
    }
    pub fn list_files(
        &self,
        conn: &mut PgConn,
        uid: &str,
        c: Option<&str>,
    ) -> Result<Vec<UserFile>> {
        let mut q = user_files::table
            .filter(user_files::user_id.eq(uid))
            .select(UserFile::as_select())
            .order(user_files::created_at.desc())
            .into_boxed();
        if let Some(c) = c {
            q = q.filter(user_files::collection.eq(c));
        }
        q.load(conn).map_err(Error::from)
    }
    pub fn delete_file(&self, conn: &mut PgConn, uid: &str, c: &str, f: &str) -> Result<()> {
        let n = diesel::delete(
            user_files::table
                .filter(user_files::user_id.eq(uid))
                .filter(user_files::collection.eq(c))
                .filter(user_files::filename.eq(f)),
        )
        .execute(conn)?;
        if n > 0 {
            let _ = fs::remove_file(self.file_path(uid, c, f));
        }
        Ok(())
    }
    pub fn file_path_on_disk(&self, uid: &str, c: &str, f: &str) -> Result<PathBuf> {
        let p = self.file_path(uid, c, f);
        if !p.exists() {
            return Err(Error::NotFound("file"));
        }
        Ok(p)
    }
}

type PgConn = diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<db::Connection>>;

fn safe_col(s: &str) -> Result<String> {
    let c = s.trim().to_lowercase();
    if c.is_empty() || c.len() > 64 || c.contains("..") || c.contains('/') || c.contains('\\') {
        return Err(Error::Invalid("bad collection".into()));
    }
    Ok(c)
}
fn safe_fn(s: &str) -> Result<String> {
    let f = s.trim().to_string();
    if f.is_empty() || f.len() > 255 || f.contains("..") || f.contains('/') || f.contains('\\') {
        return Err(Error::Invalid("bad filename".into()));
    }
    Ok(f)
}
