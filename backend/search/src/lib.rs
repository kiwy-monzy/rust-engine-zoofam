use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("tantivy: {0}")]
    Tantivy(#[from] tantivy::error::TantivyError),
    #[error("directory: {0}")]
    Directory(String),
    #[error("query: {0}")]
    Query(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, SearchError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub description: String,
    pub url: Option<String>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDoc {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub collection: Option<String>,
    pub description: String,
    pub url: Option<String>,
    pub meta: Option<serde_json::Value>,
}

pub struct SearchIndex {
    index: Index,
    reader: IndexReader,
    #[allow(dead_code)]
    schema: Schema,
    f_id: Field,
    f_kind: Field,
    f_name: Field,
    f_collection: Field,
    f_description: Field,
    f_url: Field,
    f_meta: Field,
}

impl SearchIndex {
    pub fn open(dir: impl Into<PathBuf>) -> Result<Self> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)?;

        let mut schema_builder = Schema::builder();
        let f_id = schema_builder.add_text_field("id", STRING | STORED);
        let f_kind = schema_builder.add_text_field("kind", STRING | STORED);
        let f_name = schema_builder.add_text_field("name", TEXT | STORED);
        let f_collection = schema_builder.add_text_field("collection", STRING | STORED);
        let f_description = schema_builder.add_text_field("description", TEXT | STORED);
        let f_url = schema_builder.add_text_field("url", STORED);
        let f_meta = schema_builder.add_text_field("meta", STORED);
        let schema = schema_builder.build();

        let index = Index::open_or_create(
            tantivy::directory::MmapDirectory::open(&dir)
                .map_err(|e| SearchError::Directory(e.to_string()))?,
            schema.clone(),
        )?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?;

        Ok(Self {
            index,
            reader,
            schema,
            f_id,
            f_kind,
            f_name,
            f_collection,
            f_description,
            f_url,
            f_meta,
        })
    }

    pub fn index_doc(&self, doc: &IndexDoc) -> Result<()> {
        let mut writer: IndexWriter = self.index.writer(50_000_000)?;
        let meta_str = doc
            .meta
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .unwrap_or_default();

        writer.add_document(doc!(
            self.f_id => doc.id.as_str(),
            self.f_kind => doc.kind.as_str(),
            self.f_name => doc.name.as_str(),
            self.f_collection => doc.collection.as_deref().unwrap_or(""),
            self.f_description => doc.description.as_str(),
            self.f_url => doc.url.as_deref().unwrap_or(""),
            self.f_meta => meta_str.as_str(),
        ))?;
        writer.commit()?;
        Ok(())
    }

    pub fn index_batch(&self, docs: &[IndexDoc]) -> Result<()> {
        let mut writer: IndexWriter = self.index.writer(50_000_000)?;
        for doc in docs {
            let meta_str = doc
                .meta
                .as_ref()
                .map(|v| serde_json::to_string(v).unwrap_or_default())
                .unwrap_or_default();

            writer.add_document(doc!(
                self.f_id => doc.id.as_str(),
                self.f_kind => doc.kind.as_str(),
                self.f_name => doc.name.as_str(),
                self.f_collection => doc.collection.as_deref().unwrap_or(""),
                self.f_description => doc.description.as_str(),
                self.f_url => doc.url.as_deref().unwrap_or(""),
                self.f_meta => meta_str.as_str(),
            ))?;
        }
        writer.commit()?;
        Ok(())
    }

    pub fn delete_by_kind(&self, kind: &str) -> Result<()> {
        let mut writer: IndexWriter = self.index.writer(50_000_000)?;
        writer.delete_term(tantivy::Term::from_field_text(self.f_kind, kind));
        writer.commit()?;
        Ok(())
    }

    pub fn delete_doc(&self, id: &str) -> Result<()> {
        let mut writer: IndexWriter = self.index.writer(50_000_000)?;
        writer.delete_term(tantivy::Term::from_field_text(self.f_id, id));
        writer.commit()?;
        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.f_name, self.f_description, self.f_collection],
        );
        let query = query_parser
            .parse_query(query_str)
            .map_err(|e| SearchError::Query(e.to_string()))?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (_score, doc_addr) in top_docs {
            if let Ok(doc) = searcher.doc::<tantivy::TantivyDocument>(doc_addr) {
                let get = |f: Field| -> String {
                    doc.get_first(f)
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                };
                let get_opt = |f: Field| -> Option<String> {
                    doc.get_first(f)
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                };
                let meta_str = get(self.f_meta);
                let meta: Option<serde_json::Value> = if meta_str.is_empty() {
                    None
                } else {
                    serde_json::from_str(&meta_str).ok()
                };

                results.push(SearchResult {
                    id: get(self.f_id),
                    kind: get(self.f_kind),
                    name: get(self.f_name),
                    description: get(self.f_description),
                    url: get_opt(self.f_url),
                    meta,
                });
            }
        }
        Ok(results)
    }

    pub fn clear(&self) -> Result<()> {
        let mut writer: IndexWriter = self.index.writer(50_000_000)?;
        writer.delete_all_documents()?;
        writer.commit()?;
        Ok(())
    }
}
