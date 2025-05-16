use anyhow::Result;
use mvn::run::run;

#[tokio::main]
pub async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_file(false)
        .with_level(true)
        .with_line_number(false)
        .with_target(true)
        .init();

    run().await
}
