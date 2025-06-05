use anyhow::Result;
use mvn::cli::Cli;

#[tokio::main]
pub async fn main() -> Result<()> {
    Cli::read().run().await
}
