//! Utility functions for generating random bytes, which can be useful for generating
//! [`SecretKey`](input/struct.SecretKey.html)s, for example.
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::{rngs::OsRng, TryRngCore};

use crate::{Error, ErrorKind};

/// A utility function for generating cryptographically-secure random bytes. A quick glance at
/// this function's source should give you a good idea of what the function is doing.
pub fn generate_random_bytes(len: u32) -> Result<Vec<u8>, Error> {
    let mut bytes = vec![0u8; len as usize];
    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|e| Error::new(ErrorKind::OsRngError).add_context(format!("{}", e)))?;
    Ok(bytes)
}

/// A utility function for generating a cryptographically-secure, random, base64-encoded string
/// based on
/// [standard base64 encoding](https://docs.rs/base64/0.9.1/base64/constant.STANDARD.html).
/// A quick glance at this function's source should give you a good idea of what the function
/// is doing.
pub fn generate_random_base64_encoded_string(len: u32) -> Result<String, Error> {
    let mut bytes = vec![0u8; len as usize];
    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|e| Error::new(ErrorKind::OsRngError).add_context(format!("{}", e)))?;
    let output = STANDARD.encode(&bytes);
    Ok(output)
}

/// A utility function for generating a cryptographically-secure, random, base64-encoded string
/// based on a custom base64 encoding (e.g. a
/// [url-safe encoding](https://docs.rs/base64/0.9.1/base64/constant.URL_SAFE.html)).
/// A quick glance at this function's source should give you a good idea of what the
/// function is doing.
pub fn generate_random_base64_encoded_string_config(
    len: u32,
    config: impl Engine,
) -> Result<String, Error> {
    let mut bytes = vec![0u8; len as usize];
    OsRng
        .try_fill_bytes(&mut bytes)
        .map_err(|e| Error::new(ErrorKind::OsRngError).add_context(format!("{}", e)))?;
    let output = config.encode(&bytes);
    Ok(output)
}
