//! Find unused functions in Godot GDScript codebases.

pub mod cli;
pub mod scanner;

pub use cli::{display_path, run, Args};
pub use scanner::{
    default_is_test_path, find_only_test_referenced_functions, find_unused_functions,
    iter_gd_files, iter_tscn_files, scan_directory, FunctionDef, ScanResult,
};
