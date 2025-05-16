use super::{
    hashes::{get_md5, get_sha1, get_sha256, get_sha512},
    types::FILE_TYPES,
};
use anyhow::Result;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::files)]
pub struct MavenFileIn {
    pub path: String,
    pub size: i64,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::deleted_files)]
pub struct DeletedMavenFileIn {
    pub path: String,
    pub size: i64,
    pub deleted: NaiveDateTime,
    pub md5: String,
    pub sha1: String,
    pub sha256: String,
    pub sha512: String,
    pub kind: String,
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
