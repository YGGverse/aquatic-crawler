pub fn error(e: &anyhow::Error) {
    eprintln!("[{}] [error] {e}", now())
}

pub fn info(message: String) {
    eprintln!("[{}] [info] {message}", now())
}

fn now() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
