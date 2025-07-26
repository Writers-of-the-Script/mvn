use crate::{db::DbPool, s3::S3Config};
use anyhow::Result;
use axum::body::Body;
use object_store::aws::{AmazonS3, AmazonS3Builder};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct RouteContext {
    pub storage: Arc<AmazonS3>,
    pub pool: DbPool,
    pub tx: UnboundedSender<(Body, String)>,
}

impl RouteContext {
    pub async fn create(
        s3: S3Config,
        conn: DbPool,
        tx: UnboundedSender<(Body, String)>,
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
        })
    }
}
