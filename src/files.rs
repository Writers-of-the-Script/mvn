use crate::{
    db::PoolConn,
    hashes::{get_md5, get_sha1, get_sha256, get_sha512},
    models::{MavenFile, MavenFileIn},
    schema::files,
    types::FILE_TYPES,
};
use anyhow::{Result, anyhow};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, delete, insert_into};
use diesel_async::RunQueryDsl;
use object_store::{ObjectStore, local::LocalFileSystem};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct FileManager;

impl FileManager {
    pub async fn get_file(&self, path: impl AsRef<str>, conn: &mut PoolConn) -> Result<MavenFile> {
        let path = path.as_ref();

        Ok(files::table
            .filter(files::path.eq_any(vec![format!("/{}", path), path.into()]))
            .select(MavenFile::as_select())
            .first(conn)
            .await?)
    }

    pub async fn has_file(&self, path: impl AsRef<str>, conn: &mut PoolConn) -> bool {
        self.get_file(path, conn).await.is_ok()
    }

    pub async fn delete_file(
        &self,
        path: impl AsRef<str>,
        conn: &mut PoolConn,
    ) -> Result<MavenFile> {
        let path = path.as_ref();

        Ok(delete(files::table)
            .filter(files::path.eq_any(vec![format!("/{}", path), path.into()]))
            .returning(MavenFile::as_returning())
            .get_result(conn)
            .await?)
    }

    pub async fn get_all_files(&self, conn: &mut PoolConn) -> Result<HashMap<String, MavenFile>> {
        Ok(HashMap::from_iter(
            files::table
                .select(MavenFile::as_select())
                .get_results(conn)
                .await?
                .into_iter()
                .map(|it| (it.path.clone(), it)),
        ))
    }

    pub async fn upload(&self, file: MavenFileIn, conn: &mut PoolConn) -> Result<MavenFile> {
        Ok(insert_into(files::table)
            .values(file)
            .returning(MavenFile::as_returning())
            .get_result(conn)
            .await?)
    }
}

impl MavenFileIn {
    pub async fn new(path: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> Result<Self> {
        let bytes = bytes.as_ref();

        let md5_bytes = bytes.to_vec();
        let sha1_bytes = bytes.to_vec();
        let sha256_bytes = bytes.to_vec();
        let sha512_bytes = bytes.to_vec();

        let md5_task = tokio::task::spawn_blocking(move || get_md5(md5_bytes));
        let sha1_task = tokio::task::spawn_blocking(move || get_sha1(sha1_bytes));
        let sha256_task = tokio::task::spawn_blocking(move || get_sha256(sha256_bytes));
        let sha512_task = tokio::task::spawn_blocking(move || get_sha512(sha512_bytes));

        let mut kind = infer::get(bytes)
            .map(|it| FILE_TYPES.get(it.extension()).map(|it| it.to_string()))
            .flatten()
            .unwrap_or("File".into());

        if kind == *FILE_TYPES.get("zip").unwrap() && path.as_ref().ends_with(".jar") {
            kind = FILE_TYPES.get("jar").unwrap().to_string();
        }

        Ok(Self {
            path: path.as_ref().into(),
            size: bytes.len() as i64,
            md5: md5_task.await?,
            sha1: sha1_task.await?,
            sha256: sha256_task.await?,
            sha512: sha512_task.await?,
            kind,
        })
    }
}

impl MavenFile {
    pub async fn get_bytes(&self, store: &Arc<LocalFileSystem>) -> Result<Vec<u8>> {
        Ok(store
            .get(&self.md5.clone().into())
            .await?
            .bytes()
            .await?
            .to_vec())
    }

    pub async fn get_content_size(&self, store: &Arc<LocalFileSystem>) -> Result<u64> {
        Ok(store.head(&self.md5.clone().into()).await?.size)
    }

    pub fn routes(&self) -> Vec<String> {
        vec![
            self.path.clone(),
            format!("{}.md5", self.path),
            format!("{}.sha1", self.path),
            format!("{}.sha256", self.path),
            format!("{}.sha512", self.path),
        ]
    }

    pub async fn get_content(
        &self,
        path: impl AsRef<str>,
        store: &Arc<LocalFileSystem>,
    ) -> Result<Vec<u8>> {
        let path = path.as_ref();

        if path == format!("{}.md5", self.path) {
            Ok(self.md5.as_bytes().to_vec())
        } else if path == format!("{}.sha1", self.path) {
            Ok(self.sha1.as_bytes().to_vec())
        } else if path == format!("{}.sha256", self.path) {
            Ok(self.sha256.as_bytes().to_vec())
        } else if path == format!("{}.sha512", self.path) {
            Ok(self.sha512.as_bytes().to_vec())
        } else if path == self.path {
            Ok(self.get_bytes(store).await?)
        } else {
            Err(anyhow!("404 Not Found"))
        }
    }

    pub async fn get_size(
        &self,
        path: impl AsRef<str>,
        store: &Arc<LocalFileSystem>,
    ) -> Result<u64> {
        let path = path.as_ref();

        if path == format!("{}.md5", self.path) {
            Ok(self.md5.len() as u64)
        } else if path == format!("{}.sha1", self.path) {
            Ok(self.sha1.len() as u64)
        } else if path == format!("{}.sha256", self.path) {
            Ok(self.sha256.len() as u64)
        } else if path == format!("{}.sha512", self.path) {
            Ok(self.sha512.len() as u64)
        } else if path == self.path {
            Ok(self.get_content_size(store).await?)
        } else {
            Err(anyhow!("404 Not Found"))
        }
    }

    pub fn get_hash(&self, alg: impl AsRef<str>) -> Result<String> {
        match alg.as_ref() {
            "md5" => Ok(self.md5.clone()),
            "sha1" => Ok(self.sha1.clone()),
            "sha256" => Ok(self.sha256.clone()),
            "sha512" => Ok(self.sha512.clone()),
            it => Err(anyhow!("Unknown hash algorithm: {it}")),
        }
    }
}
