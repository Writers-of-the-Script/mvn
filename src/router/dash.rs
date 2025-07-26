use anyhow::Result;
use askama::Template;
use diesel::{QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use itertools::Itertools;

use crate::{
    cx::RouteContext,
    router::stats::InstanceStats,
    schema::{token_paths, tokens},
    tokens::{
        models::{MavenToken, MavenTokenPath},
        perms::MavenTokenPermissions,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HumanTokenPath {
    pub path: String,
    pub permission: String,
}

impl Into<HumanTokenPath> for MavenTokenPath {
    fn into(self) -> HumanTokenPath {
        HumanTokenPath {
            path: self.path,
            permission: match MavenTokenPermissions::from_value(self.permission) {
                Ok(MavenTokenPermissions::Read) => "Read",
                Ok(MavenTokenPermissions::Write) => "Write",
                Ok(MavenTokenPermissions::ReadWrite) => "Read/Write",
                _ => "Unknown",
            }
            .into(),
        }
    }
}

#[derive(Template)]
#[template(path = "admin.html")]
pub struct AdminDashboard {
    pub master_key: String,
    pub tokens: Vec<(MavenToken, Vec<HumanTokenPath>)>,
    pub stats: InstanceStats,
}

impl AdminDashboard {
    pub async fn get(master_key: String, cx: &RouteContext) -> Result<Self> {
        let tokens = tokens::table
            .left_join(token_paths::table)
            .select((
                MavenToken::as_select(),
                Option::<MavenTokenPath>::as_select(),
            ))
            .load::<(MavenToken, Option<MavenTokenPath>)>(&mut cx.pool.get().await?)
            .await?
            .into_iter()
            .into_group_map()
            .into_iter()
            .map(|(token, paths)| {
                (
                    token,
                    paths
                        .into_iter()
                        .filter_map(|it| it)
                        .map(Into::into)
                        .collect_vec(),
                )
            })
            .sorted_by_cached_key(|it| it.0.created)
            .collect_vec();

        Ok(Self {
            master_key,
            tokens,
            stats: cx.stats().await?,
        })
    }
}
