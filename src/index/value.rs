use chrono::{DateTime, Utc};

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
    value.map(crop)
}

fn filter_list(value: Option<Vec<(String, u64)>>) -> Option<Vec<(String, u64)>> {
    value.map(|f| {
        f.into_iter()
            .map(|(n, l)| (crop(sanitize(&n)), l))
            .collect()
    })
}

/// Crop long values (prevents unexpected memory pool usage)
fn crop(value: String) -> String {
    const L: usize = 125; // + 3 bytes for `...` offset, 128 max @TODO optional
    if value.len() > L {
        format!("{}...", sanitize(&value[..L]))
    } else {
        value
    }
}

/// Strip tags & bom chars from string
fn sanitize(value: &str) -> String {
    use voca_rs::strip::*;
    strip_tags(&strip_bom(value))
}
