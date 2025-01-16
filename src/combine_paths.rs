pub fn combine_paths(prefix: &str, nested: &str) -> String {
    let prefix = prefix.trim_end_matches('/');
    let nested = nested.trim_start_matches('/');

    // If both are empty or "/", just return "/"
    let prefix_is_root = prefix.is_empty() || prefix == "/";
    let nested_is_root = nested.is_empty() || nested == "/";

    match (prefix_is_root, nested_is_root) {
        (true, true) => "/".to_string(),
        (true, false) => format!("/{}", nested),
        (false, true) => prefix.to_string(),
        (false, false) => format!("{}/{}", prefix, nested),
    }
}
