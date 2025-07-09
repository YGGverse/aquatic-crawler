use chrono::{DateTime, Utc};
use voca_rs::Voca;

/// The `Index` value
pub struct Value {
    pub time: DateTime<Utc>,
    pub node: u64,
    // Isolate by applying internal filter on value set
    size: Option<u64>,
    name: Option<String>,
    list: Option<Vec<(String, u64)>>,
}

impl Value {
    /// Create new `Self` with current timestamp
    pub fn new(
        node: u64,
        size: Option<u64>,
        name: Option<String>,
        list: Option<Vec<(String, u64)>>,
    ) -> Self {
        Self {
            time: Utc::now(),
            node,
            size,
            list: filter_list(list),
            name: filter_name(name),
        }
    }
    /// Get reference to the safely constructed `name` member
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    /// Get reference to the safely constructed files `list` member
    pub fn list(&self) -> Option<&Vec<(String, u64)>> {
        self.list.as_ref()
    }
    /// Get reference to the safely constructed `length` member
    pub fn size(&self) -> Option<u64> {
        self.size
    }
}

fn filter_name(value: Option<String>) -> Option<String> {
    value.map(filter)
}

fn filter_list(value: Option<Vec<(String, u64)>>) -> Option<Vec<(String, u64)>> {
    value.map(|f| f.into_iter().map(|(n, l)| (filter(n), l)).collect())
}

/// Crop long values (prevents unexpected memory pool usage)
fn filter(value: String) -> String {
    const C: usize = 125; // + 3 for `...` offset, 128 chars max @TODO optional
    let s = value._strip_bom()._strip_tags();
    if s.chars().count() > C {
        return format!("{}...", s.chars().take(C).collect::<String>());
    }
    s
}
