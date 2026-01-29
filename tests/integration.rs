//! Integration tests mirroring the Python project's test_scanner and test_cli.

use std::path::Path;

use gdcf::scanner::gd_definitions::strip_string_literals;
use gdcf::scanner::util::normalize_source;
use gdcf::scanner::{
    default_is_test_path, find_function_definitions, find_function_references,
    find_only_test_referenced_functions, find_tscn_references, find_unused_functions,
    scan_directory,
};

#[test]
fn find_function_definitions_single() {
    let source = r#"
extends Node

func _ready():
    pass

func never_called():
    print("hi")
"#;
    let path = Path::new("main.gd");
    let defs = find_function_definitions(path, source);
    let names: Vec<_> = defs.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"_ready"));
    assert!(names.contains(&"never_called"));
    assert_eq!(defs.len(), 2);
}

#[test]
fn find_function_definitions_static() {
    let source = "static func helper(): pass";
    let defs = find_function_definitions(Path::new("x.gd"), source);
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "helper");
    assert!(defs[0].is_static);
}

#[test]
fn find_function_references_direct_call() {
    let source = r#"
func _ready():
    do_thing()

func do_thing():
    pass
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"do_thing"));
    assert!(names.contains(&"_ready"));
}

#[test]
fn find_function_references_connect() {
    let source = r#"
func _ready():
    $Button.pressed.connect(_on_button_pressed)

func _on_button_pressed():
    pass
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"_on_button_pressed"));
}

#[test]
fn find_function_references_call_string() {
    let source = r#"
func _ready():
    call("dynamic_method")
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"dynamic_method"));
}

#[test]
fn find_function_references_nested_call() {
    let source = r#"
func _load_settings() -> void:
    for p in players:
        player.volume_db = _linear_to_db(_get_effective_music_volume())

func _linear_to_db(linear: float) -> float:
    return linear

func _get_effective_music_volume() -> float:
    return music_volume * global_volume
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(
        names.contains(&"_get_effective_music_volume"),
        "nested call inner(outer()) should count as reference"
    );
    assert!(names.contains(&"_linear_to_db"));
}

#[test]
fn find_function_references_assigned_to_dict() {
    let source = r#"
func _ready() -> void:
    context["print"] = _console_print

func _console_print(arg) -> void:
    output.append_text(str(arg) + "\n")
"#;
    let refs = find_function_references(Path::new("a.gd"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(
        names.contains(&"_console_print"),
        "dict[key] = func should count as reference"
    );
}

#[test]
fn test_find_tscn_references() {
    let source =
        r#"[connection signal="pressed" from="Button" to="." method="_on_quit_dialog_confirmed"]"#;
    let refs = find_tscn_references(Path::new("ui.tscn"), source);
    let names: Vec<_> = refs.iter().map(|r| r.0.as_str()).collect();
    assert!(names.contains(&"_on_quit_dialog_confirmed"));
    assert_eq!(refs.len(), 1);
}

#[test]
fn find_unused_functions_bootstrap() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let script = root.join("main.gd");
    std::fs::write(
        &script,
        r#"extends Node

func _ready():
    used_helper()

func used_helper():
    print("used")

func never_called():
    print("nobody calls me")
"#,
    )
    .unwrap();
    let unused = find_unused_functions(root, None, None);
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"never_called"));
    assert!(!names.contains(&"used_helper"));
    assert!(!names.contains(&"_ready"));
}

#[test]
fn find_unused_functions_empty_project() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("main.gd"),
        "extends Node\nfunc _ready():\n    pass\n",
    )
    .unwrap();
    let unused = find_unused_functions(root, None, None);
    assert!(unused.is_empty());
}

#[test]
fn test_default_is_test_path() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("tests/foo.gd"), "").unwrap();
    std::fs::write(root.join("src/main.gd"), "").unwrap();
    assert!(default_is_test_path(root, &root.join("tests/foo.gd")));
    assert!(!default_is_test_path(root, &root.join("src/main.gd")));
    std::fs::create_dir_all(root.join("game")).unwrap();
    std::fs::write(root.join("game/logic_test.gd"), "").unwrap();
    assert!(default_is_test_path(root, &root.join("game/logic_test.gd")));
}

#[test]
fn normalize_source_empty() {
    assert_eq!(normalize_source(""), "");
}

#[test]
fn normalize_source_bom() {
    assert!(normalize_source("\u{feff}foo").starts_with('f'));
    assert_eq!(normalize_source("\u{feff}foo"), "foo");
}

#[test]
fn strip_string_literals_triple_quote() {
    let s = r#"x """ not a call() here """ y"#;
    let out = strip_string_literals(s);
    assert!(out.starts_with("x "));
    assert!(out.ends_with(" y"));
}

// --- analysis: find_only_test_referenced_functions, default_is_test_path edges ---

#[test]
fn find_only_test_referenced_functions_bootstrap() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("tests")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/main.gd"),
        r#"extends Node
func _ready():
    pass

func only_called_from_test():
    pass
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("tests/test_main.gd"),
        r#"extends Node
func test_thing():
    only_called_from_test()
"#,
    )
    .unwrap();
    let only_test = find_only_test_referenced_functions(root, None, None, None);
    let names: Vec<_> = only_test.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"only_called_from_test"));
    assert!(!names.contains(&"_ready"));
}

#[test]
fn find_only_test_referenced_functions_with_custom_is_test_path() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("main.gd"),
        r#"extends Node
func _ready():
    pass

func helper():
    pass
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("test_foo.gd"),
        r#"extends Node
func _run():
    helper()
"#,
    )
    .unwrap();
    let custom = Box::new(|p: &Path| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("test_"))
            .unwrap_or(false)
    });
    let only_test = find_only_test_referenced_functions(root, Some(custom), None, None);
    let names: Vec<_> = only_test.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"helper"));
}

#[test]
fn find_unused_functions_ref_on_def_line_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("main.gd"),
        r#"extends Node
func _ready():
    pass

func used_elsewhere():
    pass

func only_self_ref():
    only_self_ref()
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("other.gd"),
        r#"extends Node
func _ready():
    used_elsewhere()
"#,
    )
    .unwrap();
    let unused = find_unused_functions(root, None, None);
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(
        !names.contains(&"only_self_ref"),
        "self-recursive should not be unused"
    );
    assert!(!names.contains(&"used_elsewhere"));
}

#[test]
fn find_unused_functions_with_exclude_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("addons")).unwrap();
    std::fs::write(
        root.join("main.gd"),
        r#"extends Node
func _ready():
    pass
func unused_in_main():
    pass
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("addons/plugin.gd"),
        r#"extends Node
func _ready():
    pass
func unused_in_plugin():
    pass
"#,
    )
    .unwrap();
    let unused = find_unused_functions(root, None, Some(&["addons".into()]));
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"unused_in_main"));
    assert!(!names.contains(&"unused_in_plugin"));
}

#[test]
fn default_is_test_path_outside_root_returns_false() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let other = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(other.path().join("tests")).unwrap();
    std::fs::write(other.path().join("tests/foo.gd"), "").unwrap();
    assert!(!default_is_test_path(
        root,
        &other.path().join("tests/foo.gd")
    ));
}

#[test]
fn default_is_test_path_stem_test_prefix_suffix() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("test_something.gd"), "").unwrap();
    std::fs::write(root.join("something_test.gd"), "").unwrap();
    assert!(default_is_test_path(root, &root.join("test_something.gd")));
    assert!(default_is_test_path(root, &root.join("something_test.gd")));
}

// --- scan_directory with debug, iter_* ---

#[test]
fn scan_directory_with_debug_out() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("main.gd"),
        "extends Node\nfunc _ready():\n    pass\n",
    )
    .unwrap();
    let mut buf = Vec::new();
    let mut debug = Some(&mut buf as &mut dyn std::io::Write);
    let result = scan_directory(root, &mut debug, None);
    assert!(!result.definitions.is_empty());
    let out = String::from_utf8(buf).unwrap();
    assert!(out.contains("[walk]"));
}

#[test]
fn scan_directory_exclude_dirs() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::create_dir_all(root.join("addons")).unwrap();
    std::fs::write(root.join("main.gd"), "extends Node\nfunc _ready(): pass\n").unwrap();
    std::fs::write(
        root.join("addons/plugin.gd"),
        "extends Node\nfunc _ready(): pass\nfunc only_in_plugin(): pass\n",
    )
    .unwrap();
    let result = scan_directory(root, &mut None, Some(&["addons".into()]));
    let def_names: Vec<_> = result.definitions.iter().map(|d| d.name.as_str()).collect();
    assert!(def_names.contains(&"_ready"));
    assert!(!def_names.contains(&"only_in_plugin"));
}

#[test]
fn scan_directory_skips_unreadable_file() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("ok.gd"), "extends Node\nfunc _ready(): pass\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bad = root.join("bad.gd");
        std::fs::File::create(&bad).unwrap();
        std::fs::set_permissions(&bad, std::fs::Permissions::from_mode(0o000)).unwrap();
        let result = scan_directory(root, &mut None, None);
        let names: Vec<_> = result
            .definitions
            .iter()
            .map(|d| d.file.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"ok.gd".to_string()));
    }
    #[cfg(not(unix))]
    {
        let result = scan_directory(root, &mut None, None);
        assert!(!result.definitions.is_empty());
    }
}
