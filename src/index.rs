mod value;

use chrono::{Duration, Utc};
use std::collections::HashMap;
use value::Value;

/// Collect processed info hashes to skip on the next iterations (for this session)
/// * also contains optional meta info to export index as RSS or any other format
pub struct Index {
    index: HashMap<String, Value>,
    /// Removes outdated values from `index` on `Self::refresh` action
    timeout: Option<Duration>,
    /// Track index changes to prevent extra disk write operations (safe SSD life)
    /// * useful in the static RSS feed generation case, if enabled
    is_changed: bool,
    /// Store the index value in memory only when it is in use by the init options
    has_name: bool,
    has_size: bool,
    has_list: bool,
}

impl Index {
    pub fn init(
        capacity: usize,
        timeout: Option<i64>,
        has_name: bool,
        has_size: bool,
        has_list: bool,
    ) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
            timeout: timeout.map(Duration::seconds),
            has_size,
            has_name,
            has_list,
            is_changed: false,
        }
    }

    pub fn has(&self, infohash: &str) -> bool {
        self.index.contains_key(infohash)
    }

    pub fn is_changed(&self) -> bool {
        self.is_changed
    }

    pub fn list(&self) -> &HashMap<String, Value> {
        &self.index
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn nodes(&self) -> u64 {
        self.index.values().map(|i| i.node).sum::<u64>()
    }

    pub fn insert(
        &mut self,
        infohash: String,
        node: u64,
        size: u64,
        list: Option<Vec<(Option<String>, u64)>>,
        name: Option<String>,
    ) {
        if self
            .index
            .insert(
                infohash,
                Value::new(
                    node,
                    if self.has_size { Some(size) } else { None },
                    if self.has_name { name } else { None },
                    if self.has_list { list } else { None },
                ),
            )
            .is_none()
        {
            self.is_changed = true
        }
    }

    pub fn refresh(&mut self) {
        if let Some(timeout) = self.timeout {
            let t = Utc::now();
            self.index.retain(|_, v| t - v.time <= timeout)
        }
        self.is_changed = false
    }
}

#[test]
fn test() {
    use std::{thread::sleep, time::Duration};

    // test values auto-clean by timeout
    let mut i = Index::init(2, Some(3), false, false, false);

    i.insert("h1".to_string(), 0, 0, None, None);
    sleep(Duration::from_secs(1));
    i.insert("h2".to_string(), 0, 0, None, None);

    i.refresh();
    assert_eq!(i.len(), 2);

    sleep(Duration::from_secs(2));
    i.refresh();
    assert_eq!(i.len(), 1)
}
