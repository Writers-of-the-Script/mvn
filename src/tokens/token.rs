use super::models::MavenToken;
use crate::cx::RouteContextInner;
use anyhow::Result;
use parking_lot::RwLockReadGuard;

impl MavenToken {
    pub async fn can_write_to<'a>(
        &self,
        cx: &RwLockReadGuard<'a, RouteContextInner>,
        path: impl AsRef<str>,
    ) -> Result<bool> {
        let path = path.as_ref();
        let paths = cx.get_token_writable_paths(&self).await?;

        Ok(paths.iter().any(|it| path.starts_with(it)))
    }

    pub async fn can_read_from<'a>(
        &self,
        cx: &RwLockReadGuard<'a, RouteContextInner>,
        path: impl AsRef<str>,
    ) -> Result<bool> {
        let path = path.as_ref();
        let paths = cx.get_token_readable_paths(&self).await?;

        Ok(paths.iter().any(|it| path.starts_with(it)))
    }
}
