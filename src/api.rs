/// Parse infohash from the source filepath,
/// decode JSON to array on success, return None if the feed file is not reachable
pub fn get(path: &str) -> Option<Vec<String>> {
    if path.contains("://") {
        todo!("URL sources yet not supported")
    }
    let s = std::fs::read_to_string(path).ok()?; // is updating?
    let r: Option<Vec<String>> = serde_json::from_str(&s).ok(); // is incomplete?
    r
}

#[test]
fn test() {
    assert!(get("test/api/0.json").is_none());
    assert!(get("test/api/1.json").is_some());
    assert!(get("test/api/2.json").is_none());
}
