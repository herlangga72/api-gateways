use crate::config::Route;

/// Holds all routes, provides lookup
pub struct RouteStore {
    /// All registered routes
    routes: Vec<Route>,
}

impl RouteStore {
    /// Create a new empty store
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Add a route
    pub fn add(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Clear all routes (for reload)
    pub fn clear(&mut self) {
        self.routes.clear();
    }

    /// Get all routes
    pub fn all(&self) -> &[Route] {
        &self.routes
    }

    /// Find matching route for a path
    /// Returns the first route where path starts with the route prefix
    pub fn find(&self, request_path: &str) -> Option<&Route> {
        for route in &self.routes {
            if request_path.starts_with(&route.path) {
                return Some(route);
            }
        }
        None
    }
}