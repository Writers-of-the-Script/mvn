use crate::{run::run, tokens::hash::set_secret};
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
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

    /// The path to the storage directory.
    #[arg(short, long, env = "STORAGE_PATH", default_value = "maven_storage")]
    pub storage: PathBuf,

    /// The secret for hashing token values.
    #[arg(short = 'S', long, env = "HASHING_SECRET")]
    pub secret: Option<String>,
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
        run(self.host, self.port, self.database_url, self.storage).await
    }
}
