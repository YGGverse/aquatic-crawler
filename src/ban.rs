use chrono::{DateTime, Local};
use librqbit::dht::Id20;
use std::{collections::HashMap, time::Duration};

pub struct Item {
    pub expires: DateTime<Local>,
    pub info_hash: String,
}

pub struct Ban {
    index: HashMap<Id20, DateTime<Local>>,
    timeout: u64,
}

impl Ban {
    pub fn init(timeout: u64, capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
            timeout,
        }
    }

    pub fn has(&self, key: &Id20) -> bool {
        self.index.contains_key(key)
    }

    pub fn total(&self) -> usize {
        self.index.len()
    }

    pub fn update(&mut self, time: DateTime<Local>) -> Vec<Item> {
        let mut b = Vec::with_capacity(self.index.len());
        self.index.retain(|i, &mut expires| {
            if time > expires {
                b.push(Item {
                    expires,
                    info_hash: i.as_string(),
                });
                false
            } else {
                true
            }
        });
        b
    }

    /// * returns expiration time
    pub fn add(&mut self, key: Id20) -> DateTime<Local> {
        let t = Local::now() + Duration::from_secs(self.index.len() as u64 * self.timeout);
        assert!(self.index.insert(key, t).is_none());
        t
    }
}
