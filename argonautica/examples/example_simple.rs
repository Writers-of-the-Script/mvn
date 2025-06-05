extern crate argonautica;
extern crate dotenv;

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

fn main() -> anyhow::Result<()> {
    let secret_key = load_secret_key()?;
    let mut hasher = Hasher::default();
    let hash = hasher
        .with_password("P@ssw0rd")
        .with_secret_key(&secret_key)
        .hash()?;
    println!("{}", &hash);

    let mut verifier = Verifier::default();
    let is_valid = verifier
        .with_hash(&hash)
        .with_password("P@ssw0rd")
        .with_secret_key(&secret_key)
        .verify()?;

    assert!(is_valid);
    Ok(())
}
