use std::sync::RwLock;
use std::collections::HashMap;

/// Statistics for rate limiting
pub struct RateLimiter {
    /// Map from client ID to request count in current window
    counts: RwLock<HashMap<String, u32>>,
    /// Maximum requests allowed per window
    limit: u32,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(limit: u32) -> Self {
        Self {
            counts: RwLock::new(HashMap::new()),
            limit,
        }
    }

    /// Check if request should be allowed
    /// Increments counter if allowed
    pub fn check(&self, client_id: &str) -> bool {
        let mut counts = self.counts.write().unwrap();
        let count = counts.entry(client_id.to_string()).or_insert(0);

        if *count >= self.limit {
            return false;
        }

        *count += 1;
        true
    }

    /// Reset all counters (call at window end)
    pub fn reset(&self) {
        let mut counts = self.counts.write().unwrap();
        counts.clear();
    }
}