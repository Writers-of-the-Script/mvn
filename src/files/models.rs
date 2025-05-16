use anyhow::{Result, anyhow};
use chrono::NaiveDateTime;
use object_store::ObjectStore;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::files)]
pub struct MavenFile {
    pub id: i32,
    pub path: String,
    pub parent: String,
    pub size: i64,
    pub uploaded: NaiveDateTime,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Identifiable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::deleted_files)]
pub struct DeletedMavenFile {
    pub id: i32,
    pub path: String,
    pub parent: String,
    pub size: i64,
    pub uploaded: NaiveDateTime,
    pub deleted: NaiveDateTime,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
}

impl MavenFile {
    pub async fn get_bytes<S: ObjectStore>(&self, store: &Arc<S>) -> Result<Vec<u8>> {
        Ok(store
            .get(&self.md5.clone().into())
            .await?
            .bytes()
            .await?
            .to_vec())
    }

    pub async fn get_content_size<S: ObjectStore>(&self, store: &Arc<S>) -> Result<u64> {
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

    pub async fn get_content<S: ObjectStore>(
        &self,
        path: impl AsRef<str>,
        store: &Arc<S>,
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

    pub async fn get_size<S: ObjectStore>(
        &self,
        path: impl AsRef<str>,
        store: &Arc<S>,
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
