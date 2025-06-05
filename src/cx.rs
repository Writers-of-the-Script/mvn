use crate::db::DbPool;
use anyhow::Result;
use axum::body::Body;
use tokio::sync::mpsc::UnboundedSender;
use object_store::local::LocalFileSystem;
use std::{fs, path::PathBuf, sync::Arc};

pub struct RouteContext {
    pub storage: Arc<LocalFileSystem>,
    pub pool: DbPool,
    pub tx: UnboundedSender<(Body, String)>,
}

impl RouteContext {
    pub async fn create(storage_path: Option<PathBuf>, conn: DbPool, tx: UnboundedSender<(Body, String)>) -> Result<Self> {
        let storage_path = storage_path.unwrap_or(PathBuf::from("maven_storage"));

        if !fs::exists(&storage_path)? {
            fs::create_dir_all(&storage_path)?;
        }

        Ok(Self {
            storage: Arc::new(LocalFileSystem::new_with_prefix(storage_path)?),
            pool: conn,
            tx,
        })
    }
}
