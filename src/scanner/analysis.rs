//! Analyze scan results: unused functions and test-only referenced functions.

use std::path::Path;

use super::constants::{is_engine_callback, is_gut_test_function};
use super::models::{FunctionDef, RefSite, ScanResult};
use super::scan::scan_directory;

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

/// Return list of function definitions that are never referenced.
/// Engine callbacks (e.g. _ready) are always considered used.
/// References on the same file:line as a definition (the def line itself) are ignored.
pub fn find_unused_functions(
    root: &Path,
    scan: Option<&ScanResult>,
    exclude_dirs: Option<&[String]>,
) -> Vec<FunctionDef> {
    let scan = match scan {
        Some(s) => s,
        None => {
            let mut debug_out = None;
            let s = scan_directory(root, &mut debug_out, exclude_dirs);
            return find_unused_functions(root, Some(&s), exclude_dirs);
        }
    };
    let def_sites: std::collections::HashSet<(PathBuf, u32, String)> = scan
        .definitions
        .iter()
        .map(|fd| {
            (
                fd.file.canonicalize().unwrap_or(fd.file.clone()),
                fd.line,
                fd.name.clone(),
            )
        })
        .collect();
    let mut unused = Vec::new();
    for fd in &scan.definitions {
        if is_engine_callback(&fd.name) {
            continue;
        }
        if is_gut_test_function(&fd.name) {
            continue;
        }
        if fd.ignore_dead_code {
            continue;
        }
        let refs: std::collections::HashSet<RefSite> = scan
            .references
            .get(&fd.name)
            .map(|s| {
                s.iter()
                    .filter(|r| {
                        let path = r.path.canonicalize().unwrap_or(r.path.clone());
                        !def_sites.contains(&(path, r.line, fd.name.clone()))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        if refs.is_empty() {
            unused.push(fd.clone());
        }
    }
    unused
}

use std::path::PathBuf;

/// Callback type: returns true if the path is considered test code.
pub type IsTestPathFn = Box<dyn Fn(&Path) -> bool>;

/// Return functions defined in main-app code that are only referenced from test code.
#[allow(clippy::type_complexity)]
pub fn find_only_test_referenced_functions(
    root: &Path,
    is_test_path: Option<IsTestPathFn>,
    scan: Option<&ScanResult>,
    exclude_dirs: Option<&[String]>,
) -> Vec<FunctionDef> {
    let is_test_path: Box<dyn Fn(&Path) -> bool> = match is_test_path {
        Some(f) => f,
        None => {
            let root = root.to_path_buf();
            Box::new(move |p| default_is_test_path(&root, p))
        }
    };
    let scan = match scan {
        Some(s) => s,
        None => {
            let mut debug_out = None;
            let s = scan_directory(root, &mut debug_out, exclude_dirs);
            return find_only_test_referenced_functions(root, None, Some(&s), exclude_dirs);
        }
    };
    let def_sites: std::collections::HashSet<(PathBuf, u32, String)> = scan
        .definitions
        .iter()
        .map(|fd| {
            (
                fd.file.canonicalize().unwrap_or(fd.file.clone()),
                fd.line,
                fd.name.clone(),
            )
        })
        .collect();
    let mut result = Vec::new();
    for fd in &scan.definitions {
        if is_engine_callback(&fd.name) {
            continue;
        }
        if is_test_path(&fd.file) {
            continue;
        }
        if fd.ignore_dead_code {
            continue;
        }
        let refs: Vec<RefSite> = scan
            .references
            .get(&fd.name)
            .map(|s| {
                s.iter()
                    .filter(|r| {
                        let path = r.path.canonicalize().unwrap_or(r.path.clone());
                        !def_sites.contains(&(path, r.line, fd.name.clone()))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        if refs.is_empty() {
            continue;
        }
        if refs.iter().all(|r| is_test_path(&r.path)) {
            result.push(fd.clone());
        }
    }
    result
}
