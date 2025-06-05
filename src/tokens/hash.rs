#![allow(static_mut_refs)]

use anyhow::{Result, anyhow};
use argonautica::{Hasher, Verifier, input::Salt};
use std::mem::MaybeUninit;
use tracing::info;

pub static mut SECRET_KEY: MaybeUninit<String> = MaybeUninit::uninit();

pub fn set_secret(secret: Option<String>) {
    unsafe {
        SECRET_KEY.write(secret.unwrap_or_else(|| {
            let mut buf = [0u8; 64];
            rand::fill(&mut buf);
            let it = buf.map(|it| format!("{:02x?}", it)).join("");
            info!(">> Your hashing secret is: {it}");
            info!(">> SAVE IT NOW, OR YOU WILL NEVER BE ABLE TO LOG IN AGAIN!");
            it
        }));
    }
}

pub fn hash_token_value(value: impl AsRef<str>) -> Result<String> {
    Ok(Hasher::new()
        .configure_memory_size(256)
        .configure_iterations(3)
        .with_salt(Salt::random(16))
        .with_password(value.as_ref())
        .with_secret_key(unsafe { SECRET_KEY.assume_init_ref() })
        .hash()
        .map_err(|e| anyhow!(e))?)
}

pub fn check_password(pass: impl AsRef<str>, hash: impl AsRef<str>) -> bool {
    Verifier::new()
        .with_password(pass.as_ref())
        .with_hash(hash)
        .with_secret_key(unsafe { SECRET_KEY.assume_init_ref() })
        .verify()
        .unwrap_or(false)
}
