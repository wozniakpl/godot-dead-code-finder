//! Find function definitions that are never referenced.

use std::path::{Path, PathBuf};

use super::super::constants::{is_engine_callback, is_gut_test_function};
use super::super::models::{FunctionDef, RefSite, ScanResult};
use super::super::scan::scan_directory;

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
