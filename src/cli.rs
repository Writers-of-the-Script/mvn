use crate::{run::run, s3::S3Config, tokens::hash::set_secret};
use anyhow::Result;
use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    EnvFilter, Layer, fmt, layer::SubscriberExt, registry, util::SubscriberInitExt,
};

/// A super cool, blazingly fast, good-looking, and secure maven repo for all your needs!
#[derive(Debug, Parser)]
#[command(name = "SuperMaven", version, about, author, long_about = None)]
pub struct Cli {
    /// The address to host the server on.
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    pub host: String,

    /// The port to host the server on.
    #[arg(short, long, default_value_t = 4000)]
    pub port: u16,

    /// The database connection string.
    #[arg(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgresql://mvn:mvn@localhost:5432/mvn"
    )]
    pub database_url: String,

    // The URL for the S3 instance.
    #[arg(long, env = "S3_URL", default_value = "None")]
    pub s3_url: Option<String>,

    /// The S3 region.
    #[arg(long, env = "S3_REGION", default_value = "us-west-1")]
    pub s3_region: String,

    /// The S3 bucket to use for storage.
    #[arg(long, env = "S3_BUCKET", default_value = "mvn")]
    pub s3_bucket: String,

    /// The ID of the S3 access key.
    #[arg(long, env = "S3_ACCESS_KEY_ID", default_value = "mvn")]
    pub s3_access_key_id: String,

    /// The secret for the S3 access key.
    #[arg(long, env = "S3_ACCESS_KEY_SECRET", default_value = "mvn")]
    pub s3_access_key_secret: String,

    /// The secret for hashing token values.
    #[arg(short = 'S', long, env = "HASHING_SECRET")]
    pub secret: Option<String>,

    /// The master key.
    #[arg(short = 'M', long, env = "MASTER_KEY")]
    pub master_key: Option<String>,
}

impl Cli {
    pub fn read() -> Self {
        #[cfg(not(miri))]
        let _ = dotenvy::dotenv();

        Self::parse()
    }

    pub async fn run(self) -> Result<()> {
        let fmt = fmt::layer()
            .with_ansi(true)
            .with_file(false)
            .with_level(true)
            .with_line_number(false)
            .with_target(true);

        let env = EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy();

        // .add_directive("tokio_postgres::connection=warn".parse().unwrap())
        // .add_directive("tokio_postgres::query=warn".parse().unwrap());

        registry().with(fmt.with_filter(env)).init();
        set_secret(self.secret);

        run(
            self.host,
            self.port,
            self.database_url,
            self.master_key,
            S3Config {
                region: self.s3_region,
                bucket: self.s3_bucket,
                access_key_id: self.s3_access_key_id,
                access_key_secret: self.s3_access_key_secret,
                url: self.s3_url,
            },
        )
        .await
    }
}
