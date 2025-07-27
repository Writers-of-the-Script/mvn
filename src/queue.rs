#![allow(static_mut_refs)]

use crate::cx::RouteContext;
use anyhow::Result;
use tokio::sync::Notify;
use std::sync::Arc;
use tracing::info;

pub async fn worker_thread(notify: Arc<Notify>, cx: Arc<RouteContext>) -> Result<()> {
    let mut conn = cx.pool.get().await?;
    
    info!("Started upload worker thread!");

    loop {
        notify.notified().await;

        debug!("Re-indexing dirs...");

        cx.index_dirs(&mut conn).await?;
    }
}
