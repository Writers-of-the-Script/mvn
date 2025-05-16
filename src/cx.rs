use crate::{
    db::DbPool,
    files::FileManager,
    hashes::HASH_TYPES,
    models::{
        MasterKey, MavenFile, MavenFileIn, MavenToken, MavenTokenIn, MavenTokenPath,
        MavenTokenPathIn, MavenTokenPermissions,
    },
    schema::{master_keys, token_paths, tokens},
};
use anyhow::{Result, anyhow};
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper,
    insert_into,
};
use diesel_async::RunQueryDsl;
use object_store::{ObjectStore, PutPayload, local::LocalFileSystem};
use parking_lot::RwLock;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

pub type RouteContext = Arc<RwLock<RouteContextInner>>;

pub struct RouteContextInner {
    pub files: FileManager,
    pub dirs: Vec<String>,
    pub dir_entries: HashMap<String, Vec<String>>,
    pub storage: Arc<LocalFileSystem>,
    pub pool: DbPool,
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
            files: Default::default(),
            dirs: Vec::new(),
            dir_entries: HashMap::new(),
            storage: Arc::new(LocalFileSystem::new_with_prefix(storage_path)?),
            pool: conn,
        };

        me.index_dirs().await?;

        Ok(me)
    }

    pub async fn index_dirs(&mut self) -> Result<()> {
        self.dirs = vec!["/".into()];
        self.dir_entries = HashMap::new();
        self.dir_entries.insert("/".into(), Vec::new());

        for (key, file) in &self
            .files
            .get_all_files(&mut self.pool.get().await?)
            .await?
        {
            let mut parts = key.split("/").map(|v| v.to_string()).collect::<Vec<_>>();

            parts.pop();

            let mut current = Vec::new();

            for part in &parts {
                current.push(part.clone());

                let cur = format!("/{}/", current.join("/")).replace("//", "/");

                if !self.dirs.contains(&cur) {
                    self.dirs.push(cur.clone());
                }

                if !self.dir_entries.contains_key(&cur) {
                    self.dir_entries.insert(cur.clone(), Vec::new());
                }

                let mut parent_list = current.clone();

                parent_list.pop();

                let parent = format!("/{}/", parent_list.join("/")).replace("//", "/");
                let entries = self.dir_entries.get_mut(&parent).unwrap();

                if !entries.contains(&cur) && cur != "/" {
                    entries.push(cur);
                }
            }

            let parent = format!("/{}/", parts.join("/")).replace("//", "/");
            let entries = self.dir_entries.get_mut(&parent).unwrap();

            for route in file.routes() {
                let route = route.split("/").last().unwrap().into();

                if !entries.contains(&route) {
                    entries.push(route);
                }
            }
        }

        Ok(())
    }

    pub fn get_path(&self, route: impl AsRef<str>) -> String {
        format!(
            "/{}",
            route
                .as_ref()
                .trim_end_matches(".md5")
                .trim_end_matches(".sha1")
                .trim_end_matches(".sha256")
                .trim_end_matches(".sha512")
        )
        .replace("//", "/")
    }

    pub async fn validate_master_key(&self, key: impl AsRef<str>) -> Result<bool> {
        Ok(master_keys::table
            .filter(master_keys::value.eq(key.as_ref()))
            .select(MasterKey::as_select())
            .get_result(&mut self.pool.get().await?)
            .await
            .is_ok())
    }

    pub async fn get_file_for_route(&self, route: impl AsRef<str>) -> Result<MavenFile> {
        self.files
            .get_file(&self.get_path(route), &mut self.pool.get().await?)
            .await
    }

    pub async fn get_file(&self, path: impl AsRef<str>) -> Result<MavenFile> {
        self.files.get_file(path, &mut self.pool.get().await?).await
    }

    pub async fn has_file(&self, path: impl AsRef<str>) -> Result<bool> {
        Ok(self.files.has_file(path, &mut self.pool.get().await?).await)
    }

    pub async fn delete_file(&self, path: impl AsRef<str>) -> Result<MavenFile> {
        self.files
            .delete_file(path, &mut self.pool.get().await?)
            .await
    }

    pub async fn create_token(&self, token: MavenTokenIn) -> Result<MavenToken> {
        Ok(insert_into(tokens::table)
            .values(token)
            .returning(MavenToken::as_returning())
            .get_result(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_token(
        &self,
        name: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<MavenToken> {
        Ok(tokens::table
            .filter(
                tokens::name
                    .eq(name.as_ref())
                    .and(tokens::value.eq(value.as_ref())),
            )
            .select(MavenToken::as_select())
            .get_result(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_token_by_name(&self, name: impl AsRef<str>) -> Result<MavenToken> {
        Ok(tokens::table
            .filter(tokens::name.eq(name.as_ref()))
            .select(MavenToken::as_select())
            .get_result(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_token_by_value(&self, value: impl AsRef<str>) -> Result<MavenToken> {
        Ok(tokens::table
            .filter(tokens::value.eq(value.as_ref()))
            .select(MavenToken::as_select())
            .get_result(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_token_paths(&self, token: &MavenToken) -> Result<Vec<MavenTokenPath>> {
        Ok(MavenTokenPath::belonging_to(token)
            .select(MavenTokenPath::as_select())
            .load(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_token_writable_paths(&self, token: &MavenToken) -> Result<Vec<String>> {
        self.get_token_paths(token).await.map(|items| {
            items
                .into_iter()
                .filter_map(
                    |it| match MavenTokenPermissions::from_value(it.permission) {
                        Ok(MavenTokenPermissions::Read) => None,
                        Ok(MavenTokenPermissions::Write) => Some(it.path),
                        Ok(MavenTokenPermissions::ReadWrite) => Some(it.path),
                        Err(_) => None,
                    },
                )
                .collect()
        })
    }

    pub async fn get_token_readable_paths(&self, token: &MavenToken) -> Result<Vec<String>> {
        self.get_token_paths(token).await.map(|items| {
            items
                .into_iter()
                .filter_map(
                    |it| match MavenTokenPermissions::from_value(it.permission) {
                        Ok(MavenTokenPermissions::Read) => Some(it.path),
                        Ok(MavenTokenPermissions::Write) => None,
                        Ok(MavenTokenPermissions::ReadWrite) => Some(it.path),
                        Err(_) => None,
                    },
                )
                .collect()
        })
    }

    pub async fn add_token_path(
        &self,
        name: impl AsRef<str>,
        path: impl AsRef<str>,
        permissions: MavenTokenPermissions,
    ) -> Result<MavenTokenPath> {
        let token = self.get_token_by_name(name).await?;
        let create = MavenTokenPathIn::new(token.id, path, permissions.value());

        Ok(insert_into(token_paths::table)
            .values(create)
            .returning(MavenTokenPath::as_returning())
            .get_result(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn upload(
        &self,
        path: impl AsRef<str>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<MavenFile> {
        let path = format!("/{}", path.as_ref()).replace("//", "/");
        let mut conn = self.pool.get().await?;

        if HASH_TYPES
            .iter()
            .any(|it| path.ends_with(&format!(".{it}")))
        {
            let alg = path.split(".").last().unwrap();
            let real = path.trim_end_matches(&format!(".{}", alg));
            let given = String::from_utf8(bytes.as_ref().to_vec())?;

            return match self.files.get_file(&real.to_owned(), &mut conn).await {
                Ok(file) => {
                    let existing = file.get_hash(alg)?;

                    if existing == given {
                        Ok(file.clone())
                    } else {
                        Err(anyhow!("Hash mismatch: expected {existing}, got {given}"))
                    }
                }

                _ => Err(anyhow!("Could not get a parent file for path: {}", path)),
            };
        }

        let bytes = bytes.as_ref().to_vec();
        let file = MavenFileIn::new(&path, &bytes).await?;

        self.storage
            .put(
                &file.md5.clone().into(),
                PutPayload::from_bytes(bytes.into()),
            )
            .await?;

        if self.has_file(&path).await? {
            self.delete_file(&path).await?;
        }

        Ok(self.files.upload(file, &mut conn).await?)
    }
}
