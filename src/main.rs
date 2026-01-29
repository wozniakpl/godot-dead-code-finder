//! CLI entrypoint for godot-dead-code-finder.

use std::path::PathBuf;

use clap::{CommandFactory, Parser};
use gdcf::scanner::{
    default_is_test_path, find_only_test_referenced_functions, find_unused_functions,
    iter_gd_files, iter_tscn_files, scan_directory,
};

#[derive(Parser)]
#[command(name = "godot-dead-code")]
#[command(about = "Find functions that are never called in a Godot GDScript codebase.")]
struct Args {
    /// Root directory to scan (default when omitted: current directory)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Only exit with code 1 if unused or test-only functions found; no output
    #[arg(short, long)]
    quiet: bool,

    /// Dir (relative to root) treated as test code; repeatable (default: tests/, test/, *_test.gd, test_*.gd)
    #[arg(long = "test-dir", value_name = "DIR")]
    test_dirs: Vec<String>,

    /// Same as --test-dir: dir (e.g. tests) whose files are test code
    #[arg(long = "tests-dir", value_name = "DIR")]
    tests_dirs: Vec<String>,

    /// Verbose: -v = summary, -vv = list every .gd and .tscn path, -vvv = log each directory traversed
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Directory name (or **/name) to exclude from scan; can be repeated (default: **/addons)
    #[arg(long = "exclude-dir", value_name = "DIR", default_values = ["**/addons"])]
    exclude_dirs: Vec<String>,

    /// Debug mode: show all references found for a specific function name
    #[arg(long, value_name = "NAME")]
    debug_function: Option<String>,
}

fn main() {
    // No args at all: show help
    if std::env::args().len() == 1 {
        let mut cmd = Args::command();
        let _ = cmd.print_help();
        std::process::exit(0);
    }

    let mut args = Args::parse();
    // Merge --tests-dir into --test-dir
    args.test_dirs.extend(args.tests_dirs);

    let path = args.path.unwrap_or_else(|| PathBuf::from("."));
    let root = path.canonicalize().unwrap_or(path);
    if !root.is_dir() {
        eprintln!("Error: not a directory: {}", root.display());
        std::process::exit(2);
    }

    let exclude_dirs = if args.exclude_dirs.is_empty() {
        vec!["**/addons".to_string()]
    } else {
        args.exclude_dirs.clone()
    };

    let is_test_path: Box<dyn Fn(&std::path::Path) -> bool> = if args.test_dirs.is_empty() {
        let root_clone = root.clone();
        Box::new(move |path: &std::path::Path| default_is_test_path(&root_clone, path))
    } else {
        let root_clone = root.clone();
        let test_dirs = args.test_dirs.clone();
        Box::new(move |path: &std::path::Path| {
            let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            for d in &test_dirs {
                let test_base = root_clone
                    .join(d)
                    .canonicalize()
                    .unwrap_or_else(|_| root_clone.join(d));
                if path.starts_with(&test_base) {
                    return true;
                }
            }
            false
        })
    };

    // -vv / -vvv: show every .gd and .tscn file found
    if args.verbose >= 2 {
        let mut debug_out: Option<&mut dyn std::io::Write> = None;
        let mut gd_paths = iter_gd_files(&root, &mut debug_out, Some(&exclude_dirs));
        gd_paths.sort_by_key(|a| a.to_string_lossy().to_lowercase());
        let mut tscn_paths = iter_tscn_files(&root, &mut debug_out, Some(&exclude_dirs));
        tscn_paths.sort_by_key(|a| a.to_string_lossy().to_lowercase());
        eprintln!("Scanning: {}", root.display());
        eprintln!("  Root (resolved): {}", root.display());
        if args.verbose >= 3 {
            eprintln!("  (above: os.walk traversal)");
        }
        eprintln!(
            "  Recursive .gd search (case-insensitive) matched {} path(s):",
            gd_paths.len()
        );
        for p in &gd_paths {
            let rel = p.strip_prefix(&root).unwrap_or(p);
            eprintln!("    {}", rel.display());
        }
        eprintln!(
            "  Recursive .tscn search (case-insensitive) matched {} path(s):",
            tscn_paths.len()
        );
        for p in &tscn_paths {
            let rel = p.strip_prefix(&root).unwrap_or(p);
            eprintln!("    {}", rel.display());
        }
        if gd_paths.is_empty() && tscn_paths.is_empty() {
            if let Ok(entries) = std::fs::read_dir(&root) {
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

    let mut debug_out: Option<&mut dyn std::io::Write> = None;
    let mut scan_opt = if args.verbose > 0 || args.debug_function.is_some() {
        Some(scan_directory(&root, &mut debug_out, Some(&exclude_dirs)))
    } else {
        None
    };

    if args.verbose >= 1 {
        if let Some(ref scan) = scan_opt {
            let mut files: Vec<_> = scan.definitions.iter().map(|fd| fd.file.clone()).collect();
            files.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
            files.dedup();
            let total_refs: usize = scan.references.values().map(|s| s.len()).sum();
            if args.verbose < 2 {
                eprintln!("Scanning: {}", root.display());
            }
            eprintln!("  Found {} .gd file(s):", files.len());
            for f in &files {
                let rel = f.strip_prefix(&root).unwrap_or(f);
                eprintln!("    {}", rel.display());
            }
            eprintln!("  Total function definitions: {}", scan.definitions.len());
            eprintln!("  Total references: {}", total_refs);
        }
    }

    // Debug mode
    if let Some(ref func_name) = args.debug_function {
        let scan = scan_opt
            .get_or_insert_with(|| scan_directory(&root, &mut debug_out, Some(&exclude_dirs)));
        eprintln!("Debug: searching for references to '{}'", func_name);
        let defs: Vec<_> = scan
            .definitions
            .iter()
            .filter(|fd| fd.name == *func_name)
            .cloned()
            .collect();
        let refs = scan.references.get(func_name).cloned().unwrap_or_default();
        eprintln!("  Definitions found: {}", defs.len());
        for fd in &defs {
            eprintln!("    {}:{}: {}", fd.file.display(), fd.line, fd.name);
        }
        eprintln!("  References found: {}", refs.len());
        let mut ref_list: Vec<_> = refs.into_iter().collect();
        ref_list.sort_by(|a, b| (a.path.as_path(), a.line).cmp(&(b.path.as_path(), b.line)));
        for r in &ref_list {
            let rel = r.path.strip_prefix(&root).unwrap_or(&r.path);
            eprintln!("    {}:{}", rel.display(), r.line);
        }
        if defs.is_empty() {
            eprintln!("  Warning: no definition found for '{}'", func_name);
        }
        if ref_list.is_empty() && !defs.is_empty() {
            eprintln!("  Warning: no references found for '{}'", func_name);
        }
        std::process::exit(0);
    }

    let unused = find_unused_functions(&root, scan_opt.as_ref(), Some(&exclude_dirs));
    let only_in_tests = find_only_test_referenced_functions(
        &root,
        Some(is_test_path),
        scan_opt.as_ref(),
        Some(&exclude_dirs),
    );

    if args.quiet {
        std::process::exit(if unused.is_empty() && only_in_tests.is_empty() {
            0
        } else {
            1
        });
    }

    if !unused.is_empty() {
        println!("Unused (never called):");
        for fd in &unused {
            println!("  {}:{}: {}", fd.file.display(), fd.line, fd.name);
        }
    }
    if !only_in_tests.is_empty() {
        println!("Only called from test code (not from main app):");
        for fd in &only_in_tests {
            println!("  {}:{}: {}", fd.file.display(), fd.line, fd.name);
        }
    }
    if unused.is_empty() && only_in_tests.is_empty() {
        println!("No unused functions found.");
    }
    if !unused.is_empty() || !only_in_tests.is_empty() {
        std::process::exit(1);
    }
}
