use chrono::{DateTime, Utc};

const NAME_MAX_LEN: usize = 125; // + 3 bytes for `...` offset @TODO optional

/// The `Index` value
pub struct Value {
    pub time: DateTime<Utc>,
    pub node: u64,
    /// Isolate by applying internal filter on value set
    name: Option<String>,
}

impl Value {
    /// Create new `Self` with current timestamp
    pub fn new(node: u64, name: Option<String>) -> Self {
        Self {
            time: Utc::now(),
            node,
            name: filter_name(name),
        }
    }
    /// Get reference to the safely constructed `name` member
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }
}

/// Prevent unexpected memory usage on store values from unknown source
fn filter_name(value: Option<String>) -> Option<String> {
    value.map(|v| {
        if v.len() > NAME_MAX_LEN {
            format!("{}...", &v[..NAME_MAX_LEN])
        } else {
            v
        }
    })
}
