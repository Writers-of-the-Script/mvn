use anyhow::Result;
use indicatif::{ParallelProgressIterator, ProgressStyle};
use once_cell::sync::Lazy;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::{
    blocking::Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use std::{env, fs};

const FILE_DIR: &str = "out/files";
const HASH_DIR: &str = "out/hashes";

pub fn basic_auth<U, P>(username: U, password: Option<P>) -> HeaderValue
where
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    use base64::prelude::BASE64_STANDARD;
    use base64::write::EncoderWriter;
    use std::io::Write;

    let mut buf = b"Basic ".to_vec();
    {
        let mut encoder = EncoderWriter::new(&mut buf, &BASE64_STANDARD);
        let _ = write!(encoder, "{username}:");
        if let Some(password) = password {
            let _ = write!(encoder, "{password}");
        }
    }
    let mut header = HeaderValue::from_bytes(&buf).expect("base64 is always valid HeaderValue");
    header.set_sensitive(true);
    header
}

const UP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .default_headers(HeaderMap::from_iter(vec![(
            AUTHORIZATION,
            basic_auth(
                env::var("UP_USER").unwrap(),
                Some(env::var("UP_PASS").unwrap()),
            ),
        )]))
        .timeout(None)
        .build()
        .unwrap()
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexData {
    pub index: Vec<String>,
    pub hashes: Vec<String>,
}

fn upload(repo: &str, file: String) -> Result<()> {
    let target = env::var("TARGET").unwrap();
    let url = format!("{target}/{repo}/{file}");
    let path = format!("{FILE_DIR}/{repo}/{file}");

    if !fs::exists(&path)? {
        println!("[WARN] Missing file: {path}");
        return Ok(());
    }

    let data = fs::read(path)?;
    let res = UP_CLIENT.put(url).body(data).send()?;

    if !res.status().is_success() {
        panic!("Error: {}", res.text()?);
    }

    Ok(())
}

fn upload_hash(repo: &str, file: String) -> Result<()> {
    let target = env::var("TARGET").unwrap();
    let url = format!("{target}/{repo}/{file}");
    let path = format!("{HASH_DIR}/{repo}/{file}");

    if !fs::exists(&path)? {
        println!("[WARN] Missing file: {path}");
        return Ok(());
    }

    let data = fs::read(path)?;
    let res = UP_CLIENT.put(url).body(data).send()?;

    if !res.status().is_success() {
        panic!("Error: {}", res.text()?);
    }

    Ok(())
}

fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let style = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    let repos = env::var("REPOS").unwrap();

    for repo in repos.split(";").collect::<Vec<_>>() {
        let data =
            serde_json::from_str::<IndexData>(&fs::read_to_string(format!("out/{repo}.json"))?)?;

        data.index
            .into_par_iter()
            .progress_with_style(style.clone())
            .with_message(format!("[{repo}] Uploading files..."))
            .for_each(|v| upload(repo, v).unwrap());

        data.hashes
            .into_par_iter()
            .progress_with_style(style.clone())
            .with_message(format!("[{repo}] Uploading hashes..."))
            .for_each(|v| upload_hash(repo, v).unwrap());
    }

    Ok(())
}
