use chrono::{DateTime, Utc};
use voca_rs::Voca;

/// The `Index` value
pub struct Value {
    pub time: DateTime<Utc>,
    pub node: u64,
    // Isolate by applying internal filter on value set
    size: Option<u64>,
    name: Option<String>,
    list: Option<Vec<(Option<String>, u64)>>,
}

impl Value {
    /// Create new `Self` with current timestamp
    pub fn new(
        node: u64,
        size: Option<u64>,
        name: Option<String>,
        list: Option<Vec<(Option<String>, u64)>>,
    ) -> Self {
        Self {
            time: Utc::now(),
            node,
            size,
            list: list.map(|f| f.into_iter().map(|(n, l)| (filter(n), l)).collect()),
            name: filter(name),
        }
    }
    /// Get reference to the safely constructed `name` member
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
    /// Get reference to the safely constructed files `list` member
    pub fn list(&self) -> Option<&Vec<(Option<String>, u64)>> {
        self.list.as_ref()
    }
    /// Get reference to the safely constructed `length` member
    pub fn size(&self) -> Option<u64> {
        self.size
    }
}

/// Strip tags and bom chars, crop long strings (prevents memory pool overload)
fn filter(value: Option<String>) -> Option<String> {
    value.map(|v| {
        const C: usize = 125; // + 3 for `...` offset, 128 chars max @TODO optional
        let s = v._strip_bom()._strip_tags();
        if s.chars().count() > C {
            return format!("{}...", s.chars().take(C).collect::<String>());
        }
        s
    })
}
