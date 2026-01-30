//! CLI: args, path display, and run logic.

use std::path::{Path, PathBuf};

use crate::scanner::{
    default_is_test_path, find_only_test_referenced_functions, find_unused_functions,
    iter_gd_files, iter_tscn_files, scan_directory, FunctionDef, ScanResult,
};
use clap::Parser;

#[derive(Parser)]
#[command(name = "godot-dead-code")]
#[command(about = "Find functions that are never called in a Godot GDScript codebase.")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Args {
    /// Root directory to scan (default when omitted: current directory)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// Only exit with code 1 if unused or test-only functions found; no output
    #[arg(short, long)]
    pub quiet: bool,

    /// Dir (relative to root) treated as test code; repeatable (default: tests/, test/, *_test.gd, test_*.gd)
    #[arg(long = "test-dir", value_name = "DIR")]
    pub test_dirs: Vec<String>,

    /// Same as --test-dir: dir (e.g. tests) whose files are test code
    #[arg(long = "tests-dir", value_name = "DIR")]
    pub tests_dirs: Vec<String>,

    /// Verbose: -v = summary, -vv = list every .gd and .tscn path, -vvv = log each directory traversed
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Directory name (or **/name) to exclude from scan; can be repeated (default: **/addons)
    #[arg(long = "exclude-dir", value_name = "DIR", default_values = ["**/addons"])]
    pub exclude_dirs: Vec<String>,

    /// Debug mode: show all references found for a specific function name
    #[arg(long, value_name = "NAME")]
    pub debug_function: Option<String>,
}

/// Format path for user-facing output: strip Windows extended path prefix `\\?\` so it displays as a normal path.
pub fn display_path(path: &Path) -> std::borrow::Cow<'_, str> {
    let s = path.to_string_lossy();
    #[cfg(windows)]
    {
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            return std::borrow::Cow::Owned(stripped.to_string());
        }
    }
    s
}

fn resolve_root(path: Option<&PathBuf>) -> Result<PathBuf, i32> {
    let path = path.cloned().unwrap_or_else(|| PathBuf::from("."));
    let root = path.canonicalize().unwrap_or(path);
    if !root.is_dir() {
        eprintln!("Error: not a directory: {}", display_path(&root));
        return Err(2);
    }
    Ok(root)
}

fn exclude_dirs(args: &Args) -> Vec<String> {
    if args.exclude_dirs.is_empty() {
        vec!["**/addons".to_string()]
    } else {
        args.exclude_dirs.clone()
    }
}

fn build_is_test_path(
    root: &PathBuf,
    test_dirs: &[String],
) -> Box<dyn Fn(&Path) -> bool + Send> {
    if test_dirs.is_empty() {
        let root = root.clone();
        Box::new(move |path: &Path| default_is_test_path(&root, path))
    } else {
        let root = root.clone();
        let test_dirs = test_dirs.to_vec();
        Box::new(move |path: &Path| {
            let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            for d in &test_dirs {
                let test_base = root
                    .join(d)
                    .canonicalize()
                    .unwrap_or_else(|_| root.join(d));
                if path.starts_with(&test_base) {
                    return true;
                }
            }
            false
        })
    }
}

fn print_verbose_file_list(root: &Path, exclude_dirs: &[String], verbose: u8) {
    let mut debug_out: Option<&mut dyn std::io::Write> = None;
    let mut gd_paths = iter_gd_files(root, &mut debug_out, Some(exclude_dirs));
    gd_paths.sort_by_key(|a| a.to_string_lossy().to_lowercase());
    let mut tscn_paths = iter_tscn_files(root, &mut debug_out, Some(exclude_dirs));
    tscn_paths.sort_by_key(|a| a.to_string_lossy().to_lowercase());
    eprintln!("Scanning: {}", display_path(root));
    eprintln!("  Root (resolved): {}", display_path(root));
    if verbose >= 3 {
        eprintln!("  (above: os.walk traversal)");
    }
    eprintln!(
        "  Recursive .gd search (case-insensitive) matched {} path(s):",
        gd_paths.len()
    );
    for p in &gd_paths {
        let rel = p.strip_prefix(root).unwrap_or(p);
        eprintln!("    {}", display_path(rel));
    }
    eprintln!(
        "  Recursive .tscn search (case-insensitive) matched {} path(s):",
        tscn_paths.len()
    );
    for p in &tscn_paths {
        let rel = p.strip_prefix(root).unwrap_or(p);
        eprintln!("    {}", display_path(rel));
    }
    if gd_paths.is_empty() && tscn_paths.is_empty() {
        if let Ok(entries) = std::fs::read_dir(root) {
            let mut top: Vec<_> = entries.flatten().map(|e| e.file_name()).collect();
            top.sort_by_key(|a| a.to_string_lossy().to_lowercase());
            eprintln!("  Top-level entries (os.listdir): {}", top.len());
            for name in top.iter().take(50) {
                let kind = if root.join(name).is_dir() {
                    "dir"
                } else {
                    "file"
                };
                eprintln!("    [{}] {}", kind, name.to_string_lossy());
            }
            if top.len() > 50 {
                eprintln!("    ... and {} more", top.len() - 50);
            }
        }
    }
}

fn print_verbose_summary(root: &Path, scan: &ScanResult, verbose: u8) {
    let mut files: Vec<_> = scan.definitions.iter().map(|fd| fd.file.clone()).collect();
    files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    files.dedup();
    let total_refs: usize = scan.references.values().map(|s| s.len()).sum();
    if verbose < 2 {
        eprintln!("Scanning: {}", display_path(root));
    }
    eprintln!("  Found {} .gd file(s):", files.len());
    for f in &files {
        let rel = f.strip_prefix(root).unwrap_or(f);
        eprintln!("    {}", display_path(rel));
    }
    eprintln!("  Total function definitions: {}", scan.definitions.len());
    eprintln!("  Total references: {}", total_refs);
}

fn run_debug_mode(
    root: &Path,
    func_name: &str,
    scan: &ScanResult,
) -> i32 {
    eprintln!("Debug: searching for references to '{}'", func_name);
    let defs: Vec<_> = scan
        .definitions
        .iter()
        .filter(|fd| fd.name == func_name)
        .cloned()
        .collect();
    let refs = scan.references.get(func_name).cloned().unwrap_or_default();
    eprintln!("  Definitions found: {}", defs.len());
    for fd in &defs {
        eprintln!("    {}:{}: {}", display_path(&fd.file), fd.line, fd.name);
    }
    eprintln!("  References found: {}", refs.len());
    let mut ref_list: Vec<_> = refs.into_iter().collect();
    ref_list.sort_by(|a, b| (a.path.as_path(), a.line).cmp(&(b.path.as_path(), b.line)));
    for r in &ref_list {
        let rel = r.path.strip_prefix(root).unwrap_or(&r.path);
        eprintln!("    {}:{}", display_path(rel), r.line);
    }
    if defs.is_empty() {
        eprintln!("  Warning: no definition found for '{}'", func_name);
    }
    if ref_list.is_empty() && !defs.is_empty() {
        eprintln!("  Warning: no references found for '{}'", func_name);
    }
    0
}

fn print_results(unused: &[FunctionDef], only_in_tests: &[FunctionDef]) -> i32 {
    if !unused.is_empty() {
        println!("Unused (never called):");
        for fd in unused {
            println!("  {}:{}: {}", display_path(&fd.file), fd.line, fd.name);
        }
    }
    if !only_in_tests.is_empty() {
        println!("Only called from test code (not from main app):");
        for fd in only_in_tests {
            println!("  {}:{}: {}", display_path(&fd.file), fd.line, fd.name);
        }
    }
    if unused.is_empty() && only_in_tests.is_empty() {
        println!("No unused functions found.");
    }
    if !unused.is_empty() || !only_in_tests.is_empty() {
        1
    } else {
        0
    }
}

pub fn run(mut args: Args) -> i32 {
    args.test_dirs.extend(args.tests_dirs.drain(..));

    let root = match resolve_root(args.path.as_ref()) {
        Ok(r) => r,
        Err(code) => return code,
    };
    let exclude_dirs = exclude_dirs(&args);
    let is_test_path = build_is_test_path(&root, &args.test_dirs);

    if args.verbose >= 2 {
        print_verbose_file_list(&root, &exclude_dirs, args.verbose);
    }

    let mut debug_out: Option<&mut dyn std::io::Write> = None;
    let scan_opt = if args.verbose > 0 || args.debug_function.is_some() {
        Some(scan_directory(&root, &mut debug_out, Some(&exclude_dirs)))
    } else {
        None
    };

    if args.verbose >= 1 {
        if let Some(ref scan) = scan_opt {
            print_verbose_summary(&root, scan, args.verbose);
        }
    }

    if let Some(ref func_name) = args.debug_function {
        let scan = scan_opt
            .unwrap_or_else(|| scan_directory(&root, &mut debug_out, Some(&exclude_dirs)));
        return run_debug_mode(&root, func_name, &scan);
    }

    let unused = find_unused_functions(&root, scan_opt.as_ref(), Some(&exclude_dirs));
    let only_in_tests = find_only_test_referenced_functions(
        &root,
        Some(is_test_path),
        scan_opt.as_ref(),
        Some(&exclude_dirs),
    );

    if args.quiet {
        return if unused.is_empty() && only_in_tests.is_empty() {
            0
        } else {
            1
        };
    }

    print_results(&unused, &only_in_tests)
}
