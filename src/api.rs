/// Parse infohash from the source filepath,
/// decode JSON to array on success
pub fn infohashes(path: &str) -> anyhow::Result<Vec<String>> {
    let mut f = std::fs::File::open(path)?;
    let mut s = String::new();

    use std::io::Read;
    f.read_to_string(&mut s)?;

    let r: Vec<String> = serde_json::from_str(&s)?;

    Ok(r)
}
