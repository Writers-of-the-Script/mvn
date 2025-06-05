#![allow(static_mut_refs)]

use crate::cx::RouteContext;
use anyhow::Result;
use axum::body::Body;
use tokio::sync::mpsc::UnboundedReceiver;
use http_body_util::BodyExt;
use std::sync::Arc;
use tracing::info;

pub async fn worker_thread(mut rx: UnboundedReceiver<(Body, String)>, cx: Arc<RouteContext>) -> Result<()> {
    let mut conn = cx.pool.get().await?;
    
    info!("Started upload worker thread!");

    loop {
        let Some((body, path)) = rx.recv().await else {
            break;
        };

        info!("Uploading {path}...");

        debug!("Collecting bytes...");

        let collected = body.collect().await?;

        debug!("Uploading to S3 and DB...");

        cx.upload_inner(&path, collected.to_bytes(), &mut conn)
            .await.unwrap();

        debug!("Re-indexing dirs...");

        cx.index_dirs(&mut conn).await?;

        info!("Successfully uploaded {path}!");
    }

    Ok(())
}
