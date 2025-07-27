use crate::{db::DbPool, files::models_in::MavenFileIn, s3::S3Config};
use anyhow::Result;
use chrono::{DateTime, Utc};
use object_store::aws::{AmazonS3, AmazonS3Builder};
use tokio::sync::mpsc::Sender;
use std::sync::Arc;

pub struct RouteContext {
    pub storage: Arc<AmazonS3>,
    pub pool: DbPool,
    pub tx: Sender<MavenFileIn>,
    pub start_time: DateTime<Utc>,
}

impl RouteContext {
    pub async fn create(
        s3: S3Config,
        conn: DbPool,
        tx: Sender<MavenFileIn>,
    ) -> Result<Self> {
        let mut builder = AmazonS3Builder::new()
            .with_region(s3.region)
            .with_bucket_name(s3.bucket)
            .with_access_key_id(s3.access_key_id)
            .with_secret_access_key(s3.access_key_secret);

        if let Some(url) = s3.url {
            if url.starts_with("http:") {
                builder = builder.with_allow_http(true);
            }

            builder = builder.with_endpoint(url);
        }

        Ok(Self {
            storage: Arc::new(builder.build()?),
            pool: conn,
            tx,
            start_time: Utc::now(),
        })
    }
}
