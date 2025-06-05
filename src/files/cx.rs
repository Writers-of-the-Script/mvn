use super::{hashes::HASH_TYPES, models::MavenFile, models_in::MavenFileIn};
use crate::{
    cx::RouteContext,
    schema::files,
};
use anyhow::{Result, anyhow};
use axum::body::Body;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, delete, insert_into, pg::Pg};
use diesel_async::{AsyncConnection, RunQueryDsl};
use object_store::{ObjectStore, PutPayload};
use std::collections::HashMap;

impl RouteContext {
    pub async fn get_file_for_route(&self, route: impl AsRef<str>) -> Result<MavenFile> {
        self.get_file(&self.get_path(route)).await
    }

    pub async fn get_file(&self, path: impl AsRef<str>) -> Result<MavenFile> {
        Ok(self
            .get_file_inner(path, &mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_file_inner(
        &self,
        path: impl AsRef<str>,
        conn: &mut impl AsyncConnection<Backend = Pg>,
    ) -> Result<MavenFile> {
        let path = path.as_ref();

        Ok(files::table
            .filter(files::path.eq(path))
            .select(MavenFile::as_select())
            .first(conn)
            .await?)
    }

    pub async fn has_file(&self, path: impl AsRef<str>) -> bool {
        self.get_file(path).await.is_ok()
    }

    pub async fn has_file_inner(
        &self,
        path: impl AsRef<str>,
        conn: &mut impl AsyncConnection<Backend = Pg>,
    ) -> bool {
        self.get_file_inner(path, conn).await.is_ok()
    }

    pub async fn delete_file(&self, path: impl AsRef<str>) -> Result<MavenFile> {
        Ok(self
            .delete_file_inner(path, &mut self.pool.get().await?)
            .await?)
    }

    pub async fn delete_file_inner(
        &self,
        path: impl AsRef<str>,
        conn: &mut impl AsyncConnection<Backend = Pg>,
    ) -> Result<MavenFile> {
        let path = path.as_ref();

        Ok(delete(files::table)
            .filter(files::path.eq(path))
            .returning(MavenFile::as_returning())
            .get_result(conn)
            .await?)
    }

    pub async fn get_all_files(&self) -> Result<HashMap<String, MavenFile>> {
        Ok(self
            .get_all_files_inner(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_all_files_inner(
        &self,
        conn: &mut impl AsyncConnection<Backend = Pg>,
    ) -> Result<HashMap<String, MavenFile>> {
        Ok(HashMap::from_iter(
            files::table
                .select(MavenFile::as_select())
                .get_results(conn)
                .await?
                .into_iter()
                .map(|it| (it.path.clone(), it)),
        ))
    }

    pub async fn upload(
        &self,
        path: impl AsRef<str>,
        bytes: impl AsRef<[u8]>,
    ) -> Result<MavenFile> {
        Ok(self
            .upload_inner(path, bytes, &mut self.pool.get().await?)
            .await?)
    }

    pub async fn upload_inner(
        &self,
        path: impl AsRef<str>,
        bytes: impl AsRef<[u8]>,
        conn: &mut impl AsyncConnection<Backend = Pg>,
    ) -> Result<MavenFile> {
        let path = format!("/{}", path.as_ref()).replace("//", "/");

        if HASH_TYPES
            .iter()
            .any(|it| path.ends_with(&format!(".{it}")))
        {
            let alg = path.split(".").last().unwrap();
            let real = path.trim_end_matches(&format!(".{}", alg));
            let given = String::from_utf8(bytes.as_ref().to_vec())?;

            return match self.get_file_inner(&real.to_owned(), conn).await {
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

        debug!("Checking for existing record...");

        if self.has_file_inner(&file.path, conn).await {
            debug!("Deleting existing record...");

            self.delete_file_inner(&file.path, conn).await?;
        }

        debug!("Inserting into database...");

        Ok(insert_into(files::table)
            .values(file)
            .returning(MavenFile::as_returning())
            .get_result(conn)
            .await?)
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

    pub fn queue_upload(&self, body: Body, path: impl AsRef<str>) {
        self.tx.send((body, path.as_ref().into())).unwrap();
    }
}
