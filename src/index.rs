use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct Value {
    pub time: DateTime<Utc>,
    pub node: u64,
    pub name: Option<String>,
}

/// Collect processed info hashes to skip on the next iterations (for this session)
/// * also contains optional meta info to export index as RSS or any other format
pub struct Index {
    index: HashMap<String, Value>,
    /// Track index changes to prevent extra disk write operations (safe SSD life)
    /// * useful in the static RSS feed generation case, if enabled.
    is_changed: bool,
}

impl Index {
    pub fn init(capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
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

    pub fn insert(&mut self, infohash: String, node: u64, name: Option<String>) {
        if self
            .index
            .insert(
                infohash,
                Value {
                    time: Utc::now(),
                    node,
                    name,
                },
            )
            .is_none()
        {
            self.is_changed = true
        }
    }

    pub fn refresh(&mut self) {
        self.is_changed = false
        // @TODO implement also index cleanup by Value timeout
    }
}
