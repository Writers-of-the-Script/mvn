use super::{
    models::{MasterKey, MavenToken, MavenTokenPath},
    models_in::{MavenTokenIn, MavenTokenPathIn},
    perms::MavenTokenPermissions,
};
use crate::{
    cx::RouteContextInner,
    schema::{master_keys, token_paths, tokens},
};
use anyhow::Result;
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper,
    insert_into,
};
use diesel_async::RunQueryDsl;

impl RouteContextInner {
    pub async fn validate_master_key(&self, key: impl AsRef<str>) -> Result<bool> {
        Ok(master_keys::table
            .filter(master_keys::value.eq(key.as_ref()))
            .select(MasterKey::as_select())
            .get_result(&mut self.pool.get().await?)
            .await
            .is_ok())
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
}
