//! Orchestrate directory scanning: .gd definitions/references and .tscn references.

use std::io::Write;
use std::path::Path;

use super::files::{iter_gd_files, iter_tscn_files};
use super::gd_definitions::find_function_definitions;
use super::gd_references::find_function_references;
use super::models::ScanResult;
use super::tscn::find_tscn_references;
use super::util::normalize_source;

/// Read file and normalize for parsing (replace replacement char, normalize line endings/BOM).
fn read_file_normalized(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let text = text.replace('\u{fffd}', "?"); // replace invalid UTF-8 like Python errors="replace"
    Some(normalize_source(&text))
}

/// Scan a directory for .gd and .tscn files; collect definitions from .gd and references from both.
pub fn scan_directory(
    root: &Path,
    debug_out: &mut Option<&mut dyn Write>,
    exclude_dirs: Option<&[String]>,
) -> ScanResult {
    let mut result = ScanResult::default();
    for path in iter_gd_files(root, debug_out, exclude_dirs) {
        let Some(text) = read_file_normalized(&path) else { continue };
        for fd in find_function_definitions(&path, &text) {
            result.definitions.push(fd);
        }
        for (name, line) in find_function_references(&path, &text) {
            result.add_reference(name, path.clone(), line);
        }
    }
    for path in iter_tscn_files(root, debug_out, exclude_dirs) {
        let Some(text) = read_file_normalized(&path) else { continue };
        for (name, line) in find_tscn_references(&path, &text) {
            result.add_reference(name, path.clone(), line);
        }
    }
    result
}
