//! Integration tests for scan_directory.

mod common;
use common::project;

use gdcf::scanner::scan_directory;

#[test]
fn scan_directory_with_debug_out() {
    let (_dir, root) = project(&[("main.gd", "extends Node\nfunc _ready():\n    pass\n")]);
    let mut buf = Vec::new();
    let mut debug = Some(&mut buf as &mut dyn std::io::Write);
    let result = scan_directory(&root, &mut debug, None);
    assert!(!result.definitions.is_empty());
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("[walk]"));
}

#[test]
fn scan_directory_exclude_dirs() {
    let (_dir, root) = project(&[
        ("main.gd", "extends Node\nfunc _ready(): pass\n"),
        (
            "addons/plugin.gd",
            "extends Node\nfunc _ready(): pass\nfunc only_in_plugin(): pass\n",
        ),
    ]);
    let result = scan_directory(&root, &mut None, Some(&["addons".into()]));
    let def_names: Vec<_> = result.definitions.iter().map(|d| d.name.as_str()).collect();
    assert!(def_names.contains(&"_ready"));
    assert!(!def_names.contains(&"only_in_plugin"));
}

#[test]
fn scan_directory_skips_unreadable_file() {
    let (_dir, root) = project(&[("ok.gd", "extends Node\nfunc _ready(): pass\n")]);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bad = root.join("bad.gd");
        std::fs::File::create(&bad).unwrap();
        std::fs::set_permissions(&bad, std::fs::Permissions::from_mode(0o000)).unwrap();
        let result = scan_directory(&root, &mut None, None);
        let names: Vec<_> = result
            .definitions
            .iter()
            .map(|d| d.file.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"ok.gd".to_string()));
    }
    #[cfg(not(unix))]
    {
        let result = scan_directory(&root, &mut None, None);
        assert!(!result.definitions.is_empty());
    }
}
