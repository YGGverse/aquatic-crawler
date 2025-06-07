pub fn error(e: &anyhow::Error) {
    eprintln!(
        "[{}] [error] {e}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}

pub fn info(message: String) {
    eprintln!(
        "[{}] [info] {message}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    )
}
