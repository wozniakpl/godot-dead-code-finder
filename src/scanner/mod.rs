//! Scan GDScript and .tscn files for function definitions and references.

mod analysis;
mod constants;
mod files;
pub mod gd_definitions;
mod gd_references;
mod models;
mod scan;
mod tscn;
pub mod util;

pub use analysis::{
    default_is_test_path, find_only_test_referenced_functions, find_unused_functions,
};
pub use files::{iter_gd_files, iter_tscn_files};
pub use gd_definitions::find_function_definitions;
pub use gd_references::find_function_references;
pub use models::{FunctionDef, ScanResult};
pub use scan::scan_directory;
pub use tscn::find_tscn_references;
