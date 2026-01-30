//! Data models for scan results.

use std::collections::{HashMap, HashSet};
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

    /// Set of (canonical path, line, name) for each definition site. Used to exclude def line from refs.
    pub fn def_sites(&self) -> HashSet<(PathBuf, u32, String)> {
        self.definitions
            .iter()
            .map(|fd| {
                (
                    fd.file.canonicalize().unwrap_or(fd.file.clone()),
                    fd.line,
                    fd.name.clone(),
                )
            })
            .collect()
    }

    /// Refs for `name` excluding sites that are definition lines (same file:line as a def).
    pub fn refs_excluding_def_sites(&self, name: &str, def_sites: &HashSet<(PathBuf, u32, String)>) -> Vec<RefSite> {
        self.references
            .get(name)
            .map(|s| {
                s.iter()
                    .filter(|r| {
                        let path = r.path.canonicalize().unwrap_or(r.path.clone());
                        !def_sites.contains(&(path, r.line, name.to_string()))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}
