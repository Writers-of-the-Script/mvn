#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct S3Config {
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub access_key_secret: String,
    pub url: Option<String>,
}
