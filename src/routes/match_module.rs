use crate::config::Route;

/// A helper to find matching route for a path.
/// Loops through routes looking for longest prefix match.
pub fn find_matching<'a>(route_store: &'a [Route], request_path: &str) -> Option<&'a Route> {
    let mut best: Option<&'a Route> = None;
    let mut best_len = 0;

    for route in route_store {
        if request_path.starts_with(&route.path) {
            let path_len = route.path.len();
            // Prefer longer match (more specific)
            if path_len > best_len {
                best = Some(route);
                best_len = path_len;
            }
        }
    }

    best
}