use std::time::{SystemTime, UNIX_EPOCH};

pub fn error(e: &anyhow::Error) {
    eprintln!(
        "[{}] [error] {e}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

pub fn info(message: String) {
    eprintln!(
        "[{}] [info] {message}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}
