extern crate argonautica;
extern crate dotenv;
extern crate futures;

use std::env;

use argonautica::input::SecretKey;
use argonautica::{Hasher, Verifier};

// Helper method to load the secret key from a .env file. Used in `main` below.
fn load_secret_key() -> anyhow::Result<SecretKey<'static>> {
    let dotenv_path = env::current_dir()?.join("examples").join("example.env");
    dotenv::from_path(&dotenv_path).map_err(|e| anyhow::anyhow!("{}", e))?;
    let base64_encoded_secret_key = env::var("SECRET_KEY")?;
    Ok(SecretKey::from_base64_encoded(&base64_encoded_secret_key)?)
}

async fn run() -> anyhow::Result<()> {
    let secret_key = load_secret_key()?;

    let mut hasher = Hasher::default();
    let mut verifier = Verifier::default();

    hasher
        .with_password("P@ssw0rd")
        .with_secret_key(&secret_key)
        .hash_non_blocking()
        .await
        .map(|hash| {
            println!("{}", &hash);
            verifier
                .with_hash(&hash)
                .with_password("P@ssw0rd")
                .with_secret_key(&secret_key)
                .verify_non_blocking()
        })?
        .await
        .and_then(|is_valid| {
            assert!(is_valid);
            Ok(())
        })?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .build()?
        .block_on(run())?;

    Ok(())
}
