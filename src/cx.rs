use crate::db::DbPool;
use anyhow::Result;
use object_store::local::LocalFileSystem;
use parking_lot::RwLock;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

pub type RouteContext = Arc<RwLock<RouteContextInner>>;

pub struct RouteContextInner {
    pub storage: Arc<LocalFileSystem>,
    pub pool: DbPool,
    pub dirs: Vec<String>,
    pub dir_entries: HashMap<String, Vec<String>>,
}

impl RouteContextInner {
    pub async fn create(storage_path: Option<PathBuf>, conn: DbPool) -> Result<RouteContext> {
        Ok(Arc::new(RwLock::new(Self::new(storage_path, conn).await?)))
    }

    async fn new(storage_path: Option<PathBuf>, conn: DbPool) -> Result<Self> {
        let storage_path = storage_path.unwrap_or(PathBuf::from("maven_storage"));

        if !fs::exists(&storage_path)? {
            fs::create_dir_all(&storage_path)?;
        }

        let mut me = Self {
            dirs: Vec::new(),
            dir_entries: HashMap::new(),
            storage: Arc::new(LocalFileSystem::new_with_prefix(storage_path)?),
            pool: conn,
        };

        me.index_dirs().await?;

        Ok(me)
    }
}
