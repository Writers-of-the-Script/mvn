extern crate argonautica;

use argonautica::utils;

fn main() -> anyhow::Result<()> {
    let base64_encoded_secret_key = utils::generate_random_base64_encoded_string(32)?;
    println!("{}", &base64_encoded_secret_key);
    Ok(())
}
