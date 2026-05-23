use std::sync::atomic::{AtomicU64, Ordering};

/// Simple request logger - prints to stdout
pub struct Logger {
    /// Counter for request numbers
    counter: AtomicU64,
}

impl Logger {
    /// Create a new logger
    pub fn new() -> Self {
        Self {
            counter: AtomicU64::new(1),
        }
    }

    /// Log a request received
    pub fn log_request(&self, method: &str, path: &str) {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        println!("[{}] {} {}", id, method, path);
    }

    /// Log a response sent
    pub fn log_response(&self, status: u16, path: &str) {
        println!("[resp] {} - {}", status, path);
    }
}