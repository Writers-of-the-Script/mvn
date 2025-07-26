use crate::{
    cx::RouteContext,
    db::{connect, migrate},
    queue::worker_thread,
    router::build_router,
    s3::S3Config,
    seed::seed_db,
};
use anyhow::{Result, anyhow};
use rustls::crypto::ring;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::unbounded_channel;
use tracing::info;

pub async fn run(host: impl AsRef<str>, port: u16, db: String, master_key: Option<String>, s3: S3Config) -> Result<()> {
    info!("Initializing rustls...");

    ring::default_provider()
        .install_default()
        .map_err(|err| anyhow!("{:?}", err))?;

    info!("Connecting to the database...");

    let pool = connect(Some(db.clone()))?;

    info!("Running migrations...");

    migrate(&pool).await?;

    info!("Seeding required data...");

    seed_db(&pool, master_key).await?;

    info!("Creating channel...");

    let (tx, rx) = unbounded_channel();

    info!("Building context...");

    let cx = Arc::new(RouteContext::create(s3, pool, tx).await?);

    info!("Indexing...");

    cx.index_dirs(&mut cx.pool.get().await?).await?;

    info!("Starting upload worker thread...");

    let cx_clone = Arc::clone(&cx);

    tokio::task::spawn(async move { worker_thread(rx, cx_clone).await });

    info!("Creating app...");

    let app = build_router(cx);

    info!("Binding listener...");

    let listener = TcpListener::bind(format!("{}:{}", host.as_ref(), port)).await?;

    info!("Service started on http://{}:{}", host.as_ref(), port);

    axum::serve(listener, app).await?;

    Ok(())
}
