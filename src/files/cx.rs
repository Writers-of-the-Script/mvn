use super::{hashes::HASH_TYPES, models::MavenFile, models_in::MavenFileIn};
use crate::{cx::RouteContextInner, schema::files};
use anyhow::{Result, anyhow};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, delete, insert_into};
use diesel_async::RunQueryDsl;
use object_store::{ObjectStore, PutPayload};
use std::collections::HashMap;

impl RouteContextInner {
    pub async fn get_file_for_route(&self, route: impl AsRef<str>) -> Result<MavenFile> {
        self.get_file(&self.get_path(route)).await
    }

    pub async fn get_file(&self, path: impl AsRef<str>) -> Result<MavenFile> {
        let path = path.as_ref();

        Ok(files::table
            .filter(files::path.eq_any(vec![format!("/{}", path), path.into()]))
            .select(MavenFile::as_select())
            .first(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn has_file(&self, path: impl AsRef<str>) -> bool {
        self.get_file(path).await.is_ok()
    }

    pub async fn delete_file(&self, path: impl AsRef<str>) -> Result<MavenFile> {
        let path = path.as_ref();

        Ok(delete(files::table)
            .filter(files::path.eq_any(vec![format!("/{}", path), path.into()]))
            .returning(MavenFile::as_returning())
            .get_result(&mut self.pool.get().await?)
            .await?)
    }

    pub async fn get_all_files(&self) -> Result<HashMap<String, MavenFile>> {
        Ok(HashMap::from_iter(
            files::table
                .select(MavenFile::as_select())
                .get_results(&mut self.pool.get().await?)
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
        let path = format!("/{}", path.as_ref()).replace("//", "/");

        if HASH_TYPES
            .iter()
            .any(|it| path.ends_with(&format!(".{it}")))
        {
            let alg = path.split(".").last().unwrap();
            let real = path.trim_end_matches(&format!(".{}", alg));
            let given = String::from_utf8(bytes.as_ref().to_vec())?;

            return match self.get_file(&real.to_owned()).await {
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

        if self.has_file(&path).await {
            self.delete_file(&path).await?;
        }

        Ok(insert_into(files::table)
            .values(file)
            .returning(MavenFile::as_returning())
            .get_result(&mut self.pool.get().await?)
            .await?)
    }
}
