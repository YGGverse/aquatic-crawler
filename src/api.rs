use anyhow::Result;
use librqbit::dht::Id20;

/// Try parse info-hash from the given source,
/// convert bytes to valid `InfoHash` v1 array on success.
pub async fn get(source: &str, capacity: usize) -> Result<Vec<Id20>> {
    let mut i = Vec::with_capacity(capacity);
    for c in if source.starts_with("http://") {
        reqwest::get(source).await?.bytes().await?.into()
    } else {
        tokio::fs::read(source.trim_start_matches("file://")).await?
    }
    .chunks_exact(20)
    {
        i.push(Id20::from_bytes(c)?) // v1 only
    }
    Ok(i)
}

#[tokio::test]
async fn test() {
    use std::fs;

    #[cfg(not(any(target_os = "linux", target_os = "macos",)))]
    {
        todo!()
    }

    const C: usize = 2;

    const P0: &str = "/tmp/aquatic-crawler-api-test-0.bin";
    const P1: &str = "/tmp/aquatic-crawler-api-test-1.bin";
    const P2: &str = "/tmp/aquatic-crawler-api-test-2.bin";

    fs::write(P0, vec![]).unwrap();
    fs::write(P1, vec![1; 40]).unwrap(); // 20 + 20 bytes

    assert!(get(P0, C).await.unwrap().is_empty());
    assert!(get(P1, C).await.unwrap().len() == 2);
    assert!(get(P2, C).await.is_err());

    fs::remove_file(P0).unwrap();
    fs::remove_file(P1).unwrap();
}
