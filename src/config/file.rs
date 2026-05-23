use std::fs;
use std::path::Path;
use std::collections::HashMap;
use super::types::{Config, Route};

/// Load configuration from a YAML file
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    #[derive(serde::Deserialize)]
    struct YamlConfig {
        host: String,
        port: u16,
        routes: Vec<YamlRoute>,
    }

    #[derive(serde::Deserialize)]
    struct YamlRoute {
        path: String,
        backend: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    }

    let yaml: YamlConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let routes: Vec<Route> = yaml.routes
        .into_iter()
        .map(|r| Route { path: r.path, backend: r.backend, headers: r.headers })
        .collect();

    Ok(Config::new(yaml.host, yaml.port, routes))
}

/// Validate config file syntax (like nginx -t)
pub fn validate(path: &str) -> Result<(), String> {
    load_config(path).map(|_| ())
}