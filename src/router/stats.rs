#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InstanceStats {
    pub uptime_ms: u64,
    pub uptime_str: String,
    pub folders: u64,
    pub files: u64,
    pub tokens: u64,
}
