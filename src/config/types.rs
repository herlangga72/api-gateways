// Config data structures
use std::collections::HashMap;

/// A single route - maps a path prefix to a backend server
pub struct Route {
    /// Path prefix to match (e.g., "/api/users")
    pub path: String,
    /// Backend URL to forward matching requests (e.g., "http://localhost:3001")
    pub backend: String,
    /// Custom headers to add to requests (name -> value)
    pub headers: HashMap<String, String>,
}

/// Complete gateway configuration
pub struct Config {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// List of routes
    pub routes: Vec<Route>,
}

impl Config {
    /// Create a new config
    pub fn new(host: String, port: u16, routes: Vec<Route>) -> Self {
        Self { host, port, routes }
    }
}