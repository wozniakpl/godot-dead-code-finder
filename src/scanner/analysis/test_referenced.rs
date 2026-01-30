//! Find functions only referenced from test code.

use std::path::Path;

use super::super::constants::is_engine_callback;
use super::super::models::{FunctionDef, ScanResult};
use super::super::scan::scan_directory;

use super::default_is_test_path;

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
    let def_sites = scan.def_sites();
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
        let refs = scan.refs_excluding_def_sites(&fd.name, &def_sites);
        if refs.is_empty() {
            continue;
        }
        if refs.iter().all(|r| is_test_path(&r.path)) {
            result.push(fd.clone());
        }
    }
    result
}
