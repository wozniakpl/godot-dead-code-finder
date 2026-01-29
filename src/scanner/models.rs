//! Data models for scan results.

use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;

/// A function definition in a GDScript file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDef {
    pub name: String,
    pub file: PathBuf,
    pub line: u32,
    pub is_static: bool,
    /// If true, do not report as unused (tagged with e.g. # gdcf-ignore or # TODO: dead-code).
    pub ignore_dead_code: bool,
}

/// A reference site (file path and line number).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefSite {
    pub path: PathBuf,
    pub line: u32,
}

/// Result of scanning a codebase.
#[derive(Debug, Default)]
pub struct ScanResult {
    pub definitions: Vec<FunctionDef>,
    pub references: HashMap<String, std::collections::HashSet<RefSite>>,
}

impl ScanResult {
    pub fn add_reference(&mut self, name: String, path: PathBuf, line: u32) {
        self.references
            .entry(name)
            .or_default()
            .insert(RefSite { path, line });
    }
}
