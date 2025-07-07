/// Parse infohash from the source filepath,
/// decode JSON to array on success, return None if the feed is damaged (incomplete)
pub fn infohashes(path: &str) -> anyhow::Result<Option<Vec<String>>> {
    if path.contains("://") {
        todo!("URL sources yet not supported")
    }
    let s = std::fs::read_to_string(path)?;
    let r: Option<Vec<String>> = serde_json::from_str(&s).ok();
    Ok(r)
}
