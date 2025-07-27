#![allow(static_mut_refs)]

use crate::{cx::RouteContext, files::models_in::MavenFileIn};
use anyhow::Result;
use tokio::sync::mpsc::Receiver;
use std::sync::Arc;
use tracing::info;

pub async fn worker_thread(mut rx: Receiver<MavenFileIn>, cx: Arc<RouteContext>) -> Result<()> {
    let mut conn = cx.pool.get().await?;
    
    info!("Started upload worker thread!");

    loop {
        let Some(file) = rx.recv().await else {
            break;
        };

        info!("Pushing {}...", file.path);

        cx.push_file_to_db(&file, &mut conn)
            .await.unwrap();

        debug!("Re-indexing dirs...");

        cx.index_dirs(&mut conn).await?;

        info!("Successfully uploaded {}!", file.path);
    }

    Ok(())
}
