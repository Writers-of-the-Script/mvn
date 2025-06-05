extern crate argonautica;
extern crate serde_json;

use argonautica::{Hasher, Verifier};

fn serialize_hasher() -> anyhow::Result<String> {
    let additional_data = [1u8, 2, 3, 4];
    let salt = [1u8, 2, 3, 4, 5, 6, 7, 8];
    let mut hasher = Hasher::default();
    hasher
        .with_additional_data(&additional_data[..])
        .with_password("P@ssw0rd") // note: for security reasons, password is never serialized
        .with_salt(&salt[..])
        .with_secret_key("secret"); // note: for security reasons, secret key is never serialized
    let j = serde_json::to_string_pretty(&hasher)?;
    println!("*** Serialized Hasher ***");
    println!("{}\n", &j);
    // *** Serialized Hasher ***
    // {
    //   "additionalData": [1,2,3,4],
    //   "config": {
    //     "backend": "c",
    //     "hashLen": 32,
    //     "iterations": 192,
    //     "lanes": 4,
    //     "memorySize": 4096,
    //     "optOutOfSecretKey": false,
    //     "passwordClearing": false,
    //     "secretKeyClearing": false,
    //     "threads": 4,
    //     "variant": "argon2id",
    //     "version": "_0x13"
    //   },
    //   "salt": {
    //     "deterministic": [1,2,3,4,5,6,7,8]
    //   }
    // }
    Ok(j)
}

fn deserialize_hasher(j: &str) -> anyhow::Result<argonautica::Hasher> {
    let hasher: Hasher = serde_json::from_str(&j)?;
    println!("*** Deserialized Hasher ***");
    println!("{:#?}\n", &hasher);
    // *** Deserialized Hasher ***
    // Hasher {
    //     additional_data: Some(AdditionalData([1,2,3,4])),
    //     config: HasherConfig {
    //         backend: C,
    //         cpu_pool: None,
    //         hash_len: 32,
    //         iterations: 192,
    //         lanes: 4,
    //         memory_size: 4096,
    //         opt_out_of_secret_key: false,
    //         password_clearing: false,
    //         secret_key_clearing: false,
    //         threads: 4,
    //         variant: Argon2id,
    //         version: _0x13
    //     },
    //     password: None,
    //     salt: Salt(Deterministic([1,2,3,4,5,6,7,8])),
    //     secret_key: None
    // }
    Ok(hasher)
}

fn serialize_verifier() -> anyhow::Result<String> {
    let additional_data = [1u8, 2, 3, 4];
    let mut verifier = Verifier::default();
    verifier
        .with_additional_data(&additional_data[..])
        .with_hash("$argon2id$v=19$m=4096,t=128,p=2$c29tZXNhbHQ$WwD2/wGGTuw7u4BW8sLM0Q")
        .with_password("P@ssw0rd") // note: for security reasons, password is never serialized
        .with_secret_key("secret"); // note: for security reasons, secret key is never serialized
    let j = serde_json::to_string_pretty(&verifier)?;
    println!("*** Serialized Verifier ***");
    println!("{}\n", &j);
    // *** Serialized Verifier ***
    // {
    //   "hash": {
    //     "encoded": "$argon2id$v=19$m=4096,t=128,p=2$c29tZXNhbHQ$WwD2/wGGTuw7u4BW8sLM0Q"
    //   },
    //   "hasher": {
    //     "additionalData": [1,2,3,4],
    //     "config": {
    //       "backend": "c",
    //       "hashLen": 32,
    //       "iterations": 192,
    //       "lanes": 4,
    //       "memorySize": 4096,
    //       "optOutOfSecretKey": false,
    //       "passwordClearing": false,
    //       "secretKeyClearing": false,
    //       "threads": 4,
    //       "variant": "argon2id",
    //       "version": "_0x13"
    //     },
    //     "salt": {
    //       "random": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
    //     }
    //   }
    // }
    Ok(j)
}

fn deserialize_verifier(j: &str) -> anyhow::Result<argonautica::Verifier> {
    let verifier: Verifier = serde_json::from_str(&j)?;
    println!("*** Deserialized Verifier ***");
    println!("{:#?}\n", &verifier);
    // *** Deserialized Verifier ***
    // Verifier {
    //     hash: Encoded("$argon2id$v=19$m=4096,t=128,p=2$c29tZXNhbHQ$WwD2/wGGTuw7u4BW8sLM0Q"),
    //     hasher: Hasher {
    //         additional_data: Some(AdditionalData([1,2,3,4])),
    //         config: HasherConfig {
    //             backend: C,
    //             cpu_pool: None,
    //             hash_len: 32,
    //             iterations: 192,
    //             lanes: 4,
    //             memory_size: 4096,
    //             opt_out_of_secret_key: false,
    //             password_clearing: false,
    //             secret_key_clearing: false,
    //             threads: 4,
    //             variant: Argon2id,
    //             version: _0x13
    //         },
    //         password: None,
    //         salt: Salt(Random([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])),
    //         secret_key: None
    //     }
    // }
    Ok(verifier)
}

fn main() -> anyhow::Result<()> {
    let j = serialize_hasher()?;
    let _ = deserialize_hasher(&j)?;

    let j = serialize_verifier()?;
    let _ = deserialize_verifier(&j)?;
    Ok(())
}
