//! File system traversal for .gd and .tscn files.

use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Normalize **/name or path/name to just the directory name (e.g. **/addons -> addons).
pub fn normalize_exclude_dir(pattern: &str) -> String {
    let normalized = pattern.replace('\\', "/");
    let name = normalized.trim_end_matches('/');
    name.rsplit('/').next().unwrap_or(name).to_string()
}

fn matches_extension(path: &Path, ext: &str) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.to_lowercase().ends_with(ext))
}

fn walk_files_rec(
    dir_path: &Path,
    root_path: &Path,
    exclude_dirs: &HashSet<String>,
    result: &mut Vec<PathBuf>,
    extension: &str,
    debug_out: &mut Option<&mut dyn Write>,
) {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    let read_dir = match fs::read_dir(dir_path) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            dirs.push(path);
        } else if path.is_file() {
            files.push(path);
        }
    }
    if let Some(ref mut out) = debug_out {
        let matching: Vec<_> = files
            .iter()
            .filter(|p| matches_extension(p, extension))
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        let rel = path_diff(dir_path, root_path)
            .unwrap_or_else(|| dir_path.to_string_lossy().to_string());
        let _ = writeln!(out, "  [walk] dirpath={:?} (rel={:?})", dir_path, rel);
        let dir_names: Vec<_> = dirs
            .iter()
            .map(|d| d.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        let file_names: Vec<_> = files
            .iter()
            .map(|f| f.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        let _ = writeln!(out, "  [walk]   dirs={:?}", dir_names);
        let _ = writeln!(out, "  [walk]   files={:?}", file_names);
        let _ = writeln!(out, "  [walk]   {} here={:?}", extension, matching);
    }
    for p in &files {
        if matches_extension(p, extension) {
            result.push(p.clone());
        }
    }
    for d in &dirs {
        if d.file_name()
            .and_then(|n| n.to_str())
            .is_none_or(|n| !exclude_dirs.contains(&n.to_string()))
        {
            walk_files_rec(
                d.as_path(),
                root_path,
                exclude_dirs,
                result,
                extension,
                debug_out,
            );
        }
    }
}

fn path_diff(a: &Path, b: &Path) -> Option<String> {
    let a = a.canonicalize().ok()?;
    let b = b.canonicalize().ok()?;
    let a_components: Vec<_> = a.components().collect();
    let b_components: Vec<_> = b.components().collect();
    let mut i = 0;
    while i < a_components.len() && i < b_components.len() && a_components[i] == b_components[i] {
        i += 1;
    }
    let mut result = PathBuf::new();
    for _ in i..b_components.len() {
        result.push("..");
    }
    for c in a_components.iter().skip(i) {
        result.push(c);
    }
    Some(result.to_string_lossy().to_string())
}

/// Recursively yield all files under root with the given extension (case-insensitive).
fn iter_files_by_extension(
    root: &Path,
    debug_out: &mut Option<&mut dyn Write>,
    exclude_dirs: Option<&[String]>,
    extension: &str,
) -> Vec<PathBuf> {
    let root_path = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let excluded: HashSet<String> = exclude_dirs
        .unwrap_or(&[])
        .iter()
        .map(|p| normalize_exclude_dir(p))
        .collect();
    if let Some(out) = debug_out.as_mut() {
        let _ = writeln!(out, "  [walk] root={:?}", root_path);
        let _ = writeln!(out, "  [walk] cwd={:?}", std::env::current_dir().ok());
        let mut sorted: Vec<_> = excluded.iter().collect();
        sorted.sort();
        let _ = writeln!(out, "  [walk] exclude_dirs={:?}", sorted);
        let _ = writeln!(out, "  [walk] root.is_dir()={}", root_path.is_dir());
    }
    if !root_path.is_dir() {
        return Vec::new();
    }
    let mut result = Vec::new();
    walk_files_rec(
        &root_path,
        &root_path,
        &excluded,
        &mut result,
        extension,
        debug_out,
    );
    result
}

/// Recursively yield all .gd files under root (case-insensitive).
pub fn iter_gd_files(
    root: &Path,
    debug_out: &mut Option<&mut dyn Write>,
    exclude_dirs: Option<&[String]>,
) -> Vec<PathBuf> {
    iter_files_by_extension(root, debug_out, exclude_dirs, ".gd")
}

/// Recursively yield all .tscn files under root (case-insensitive).
pub fn iter_tscn_files(
    root: &Path,
    debug_out: &mut Option<&mut dyn Write>,
    exclude_dirs: Option<&[String]>,
) -> Vec<PathBuf> {
    iter_files_by_extension(root, debug_out, exclude_dirs, ".tscn")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_exclude_dir_bare_name() {
        assert_eq!(normalize_exclude_dir("addons"), "addons");
    }

    #[test]
    fn normalize_exclude_dir_with_slash() {
        assert_eq!(normalize_exclude_dir("**/addons"), "addons");
        assert_eq!(normalize_exclude_dir("foo/addons"), "addons");
    }

    #[test]
    fn normalize_exclude_dir_trailing_slash() {
        assert_eq!(normalize_exclude_dir("addons/"), "addons");
    }

    #[test]
    fn normalize_exclude_dir_backslash() {
        assert_eq!(normalize_exclude_dir("foo\\addons"), "addons");
    }

    #[test]
    fn iter_gd_files_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let files = iter_gd_files(root, &mut None, None);
        assert!(files.is_empty());
    }

    #[test]
    fn iter_gd_files_finds_gd() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("main.gd"), "").unwrap();
        std::fs::write(root.join("readme.txt"), "").unwrap();
        let files = iter_gd_files(root, &mut None, None);
        assert_eq!(files.len(), 1);
        assert!(files[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".gd"));
    }

    #[test]
    fn iter_gd_files_exclude_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("addons")).unwrap();
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("main.gd"), "").unwrap();
        std::fs::write(root.join("addons/plugin.gd"), "").unwrap();
        std::fs::write(root.join("src/foo.gd"), "").unwrap();
        let files = iter_gd_files(root, &mut None, Some(&["addons".into()]));
        let names: Vec<_> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_str().unwrap())
            .collect();
        assert!(names.contains(&"main.gd"));
        assert!(names.contains(&"foo.gd"));
        assert!(!names.contains(&"plugin.gd"));
    }

    #[test]
    fn iter_gd_files_with_debug_out() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("a.gd"), "").unwrap();
        let mut buf = Vec::new();
        let mut debug = Some(&mut buf as &mut dyn Write);
        let files = iter_gd_files(root, &mut debug, None);
        assert_eq!(files.len(), 1);
        let out = String::from_utf8(buf).unwrap();
        assert!(out.contains("[walk]"));
        assert!(out.contains("a.gd"));
    }

    #[test]
    fn iter_gd_files_non_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let file = root.join("not_a_dir");
        std::fs::write(&file, "").unwrap();
        let files = iter_gd_files(&file, &mut None, None);
        assert!(files.is_empty());
    }

    #[test]
    fn iter_tscn_files_finds_tscn() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(root.join("scene.tscn"), "").unwrap();
        std::fs::write(root.join("main.gd"), "").unwrap();
        let files = iter_tscn_files(root, &mut None, None);
        assert_eq!(files.len(), 1);
        assert!(files[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".tscn"));
    }

    #[test]
    fn iter_tscn_files_exclude_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("addons")).unwrap();
        std::fs::write(root.join("addons/thing.tscn"), "").unwrap();
        std::fs::write(root.join("ui.tscn"), "").unwrap();
        let files = iter_tscn_files(root, &mut None, Some(&["addons".into()]));
        assert_eq!(files.len(), 1);
        assert!(files[0].file_name().unwrap().to_str().unwrap() == "ui.tscn");
    }

    #[test]
    fn iter_tscn_files_non_dir_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let file = root.join("file.txt");
        std::fs::write(&file, "").unwrap();
        let files = iter_tscn_files(&file, &mut None, None);
        assert!(files.is_empty());
    }
}
