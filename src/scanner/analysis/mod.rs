//! Analyze scan results: unused functions and test-only referenced functions.

use std::path::Path;

mod test_referenced;
mod unused;

#[allow(unused_imports)]
// re-exported for public API (Option<IsTestPathFn> in find_only_test_referenced_functions)
pub use test_referenced::{find_only_test_referenced_functions, IsTestPathFn};
pub use unused::find_unused_functions;

/// Return true if path is considered test code (under root).
/// Default: any segment is 'tests' or 'test', or filename is *_test.gd / test_*.gd.
pub fn default_is_test_path(root: &Path, path: &Path) -> bool {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let rel = match path.strip_prefix(&root) {
        Ok(r) => r,
        Err(_) => return false,
    };
    for component in rel.components() {
        if let std::path::Component::Normal(os_str) = component {
            if let Some(s) = os_str.to_str() {
                if s == "tests" || s == "test" {
                    return true;
                }
            }
        }
    }
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    stem.ends_with("_test") || stem.starts_with("test_")
}
