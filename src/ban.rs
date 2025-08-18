use chrono::{DateTime, Local};
use librqbit::dht::Id20;
use std::{collections::HashMap, time::Duration};

pub struct Item {
    pub expires: DateTime<Local>,
    pub info_hash: String,
}

pub struct Ban {
    index: HashMap<Id20, Option<DateTime<Local>>>,
    timeout: Duration,
}

impl Ban {
    pub fn init(timeout: u64, capacity: usize) -> Self {
        Self {
            index: HashMap::with_capacity(capacity),
            timeout: Duration::from_secs(timeout),
        }
    }

    pub fn get(&self, key: &Id20) -> Option<&Option<DateTime<Local>>> {
        self.index.get(key)
    }

    pub fn total(&self) -> usize {
        self.index.len()
    }

    /// * return removed `Item` details
    pub fn update(&mut self, time: DateTime<Local>) -> Vec<Item> {
        let mut b = Vec::with_capacity(self.index.len());
        self.index.retain(|i, &mut e| {
            if let Some(expires) = e
                && time > expires
            {
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

    /// Add torrent to the ban list
    ///
    /// If the `is_permanent` option is `true` - ban permanently,
    /// or **automatically** count optimal ban time based on the current index max time and timeout offset value.
    ///
    /// * return expiration time or `None` if the ban is permanent
    pub fn add(&mut self, key: Id20, is_permanent: bool) -> Option<DateTime<Local>> {
        let t = if is_permanent {
            None
        } else {
            Some(
                self.index
                    .values()
                    .max()
                    .map_or(Local::now(), |t| t.unwrap_or(Local::now()) + self.timeout),
            )
        };
        assert!(self.index.insert(key, t).is_none());
        t
    }
}
