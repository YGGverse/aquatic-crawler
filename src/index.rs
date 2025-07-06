use chrono::{DateTime, Utc};

pub struct Index {
    pub time: DateTime<Utc>,
    pub node: u64,
    pub name: Option<String>,
}
