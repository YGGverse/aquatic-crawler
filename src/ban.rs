use chrono::{DateTime, Local};
use librqbit::dht::Id20;
use std::{collections::HashMap, time::Duration};

pub struct Item {
    pub expires: DateTime<Local>,
    pub info_hash: String,
}

pub struct Ban {
    index: HashMap<Id20, DateTime<Local>>,
    timeout: Duration,
}

impl Ban {
    pub fn init(timeout: u64, capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
            timeout: Duration::from_secs(timeout),
        }
    }

    pub fn get(&self, key: &Id20) -> Option<&DateTime<Local>> {
        self.index.get(key)
    }

    pub fn total(&self) -> usize {
        self.index.len()
    }

    /// * return removed `Item` details
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

    /// * return expiration time
    pub fn add(&mut self, key: Id20) -> DateTime<Local> {
        let t = self.index.values().max().map_or(Local::now(), |t| *t) + self.timeout;
        assert!(self.index.insert(key, t).is_none());
        t
    }
}
