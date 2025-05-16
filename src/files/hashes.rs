use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha512};

pub const HASH_TYPES: &[&str] = &["md5", "sha1", "sha256", "sha512"];

pub fn get_md5(file: impl AsRef<[u8]>) -> String {
    format!("{:x}", md5::compute(file))
}

pub fn get_sha1(file: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha1::new();

    hasher.update(file);

    format!("{:x}", hasher.finalize())
}

pub fn get_sha256(file: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha256::new();

    hasher.update(file);

    format!("{:x}", hasher.finalize())
}

pub fn get_sha512(file: impl AsRef<[u8]>) -> String {
    let mut hasher = Sha512::new();

    hasher.update(file);

    format!("{:x}", hasher.finalize())
}
