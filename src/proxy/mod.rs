use std::collections::HashMap;
use std::net::TcpStream;
use std::io::{Read, Write};
use std::time::Duration;
use std::sync::RwLock;

/// Connection pool - reuses TCP connections
pub struct PoolManager {
    connections: RwLock<HashMap<String, TcpStream>>,
    timeout: Duration,
}

impl PoolManager {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
            timeout: Duration::from_secs(5),
        }
    }

    /// Get or create connection to host
    fn get_connection(&self, host: &str) -> Result<TcpStream, String> {
        // Check pool
        {
            let guard = self.connections.read().unwrap();
            if let Some(conn) = guard.get(host) {
                return Ok(conn.try_clone().map_err(|e| e.to_string())?);
            }
        }

        // New connection
        let mut conn = TcpStream::connect(host)
            .map_err(|e| e.to_string())?;
        conn.set_read_timeout(Some(self.timeout)).ok();
        conn.set_write_timeout(Some(self.timeout)).ok();

        // Add to pool
        let cloned = conn.try_clone().map_err(|e| e.to_string())?;
        self.connections.write().unwrap().insert(host.to_string(), cloned);

        Ok(conn)
    }

    /// Forward request to backend
    pub fn forward(
        &self,
        backend: &str,
        path: &str,
        headers: &HashMap<String, String>,
    ) -> Result<(), String> {
        // Parse host
        let target = backend.trim_start_matches("http://").trim_start_matches("https://");
        let host = target.split('/').next().unwrap_or(target);

        let mut conn = self.get_connection(host)?;

        // Build HTTP request
        let mut req = format!("GET {} HTTP/1.1\r\nHost: {}\r\n", path, host);
        for (k, v) in headers {
            req.push_str(&format!("{}: {}\r\n", k, v));
        }
        req.push_str("\r\n");

        // Send
        conn.write_all(req.as_bytes()).map_err(|e| e.to_string())?;

        // Read response (throw away)
        let mut buf = [0u8; 512];
        let _ = conn.read(&mut buf);

        Ok(())
    }
}

impl Default for PoolManager {
    fn default() -> Self {
        Self::new()
    }
}