//! Integration tests for find_unused_functions, find_only_test_referenced_functions, default_is_test_path.

use std::path::Path;

mod common;
use common::project;

use gdcf::scanner::{
    default_is_test_path, find_only_test_referenced_functions, find_unused_functions,
};

#[test]
fn find_unused_functions_bootstrap() {
    let (_dir, root) = project(&[(
        "main.gd",
        r#"extends Node

func _ready():
    used_helper()

func used_helper():
    print("used")

func never_called():
    print("nobody calls me")
"#,
    )]);
    let unused = find_unused_functions(&root, None, None);
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"never_called"));
    assert!(!names.contains(&"used_helper"));
    assert!(!names.contains(&"_ready"));
}

#[test]
fn find_unused_functions_empty_project() {
    let (_dir, root) = project(&[("main.gd", "extends Node\nfunc _ready():\n    pass\n")]);
    let unused = find_unused_functions(&root, None, None);
    assert!(unused.is_empty());
}

#[test]
fn find_unused_functions_respects_gdcf_ignore_tag() {
    let (_dir, root) = project(&[(
        "main.gd",
        r#"extends Node

func _ready():
    pass

func will_wire_later(): # gdcf-ignore
    pass

func actually_unused():
    pass
"#,
    )]);
    let unused = find_unused_functions(&root, None, None);
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(
        !names.contains(&"will_wire_later"),
        "tagged function should not be reported as unused"
    );
    assert!(names.contains(&"actually_unused"));
}

#[test]
fn test_default_is_test_path() {
    let (_dir, root) = project(&[
        ("tests/foo.gd", ""),
        ("src/main.gd", ""),
        ("game/logic_test.gd", ""),
    ]);
    assert!(default_is_test_path(&root, &root.join("tests/foo.gd")));
    assert!(!default_is_test_path(&root, &root.join("src/main.gd")));
    assert!(default_is_test_path(
        &root,
        &root.join("game/logic_test.gd")
    ));
}

#[test]
fn find_only_test_referenced_functions_bootstrap() {
    let (_dir, root) = project(&[
        (
            "src/main.gd",
            r#"extends Node
func _ready():
    pass

func only_called_from_test():
    pass
"#,
        ),
        (
            "tests/test_main.gd",
            r#"extends Node
func test_thing():
    only_called_from_test()
"#,
        ),
    ]);
    let only_test = find_only_test_referenced_functions(&root, None, None, None);
    let names: Vec<_> = only_test.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"only_called_from_test"));
    assert!(!names.contains(&"_ready"));
}

#[test]
fn find_only_test_referenced_functions_with_custom_is_test_path() {
    let (_dir, root) = project(&[
        (
            "main.gd",
            r#"extends Node
func _ready():
    pass

func helper():
    pass
"#,
        ),
        (
            "test_foo.gd",
            r#"extends Node
func _run():
    helper()
"#,
        ),
    ]);
    let custom = Box::new(|p: &Path| {
        p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with("test_"))
            .unwrap_or(false)
    });
    let only_test = find_only_test_referenced_functions(&root, Some(custom), None, None);
    let names: Vec<_> = only_test.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"helper"));
}

#[test]
fn find_unused_functions_ref_on_def_line_ignored() {
    let (_dir, root) = project(&[
        (
            "main.gd",
            r#"extends Node
func _ready():
    pass

func used_elsewhere():
    pass

func only_self_ref():
    only_self_ref()
"#,
        ),
        (
            "other.gd",
            r#"extends Node
func _ready():
    used_elsewhere()
"#,
        ),
    ]);
    let unused = find_unused_functions(&root, None, None);
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(
        !names.contains(&"only_self_ref"),
        "self-recursive should not be unused"
    );
    assert!(!names.contains(&"used_elsewhere"));
}

#[test]
fn find_unused_functions_tween_method_callback_not_unused() {
    let (_dir, root) = project(&[(
        "audio.gd",
        r#"extends Node
const TWEEN_FADE_AUDIO_DURATION = 0.5

func set_master_volume(volume_db: float) -> void:
    master_volume = volume_db
    master_volume_changed.emit(master_volume)

func transition_master_volume(from_volume: float, to_volume: float) -> void:
    if _fade_tween != null:
        _fade_tween.kill()
    _fade_tween = create_tween()
    _fade_tween.tween_method(set_master_volume, from_volume, to_volume, TWEEN_FADE_AUDIO_DURATION)
"#,
    )]);
    let unused = find_unused_functions(&root, None, None);
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(
        !names.contains(&"set_master_volume"),
        "set_master_volume used as tween_method callback should not be reported as unused"
    );
}

#[test]
fn find_unused_functions_with_exclude_dirs() {
    let (_dir, root) = project(&[
        (
            "main.gd",
            r#"extends Node
func _ready():
    pass
func unused_in_main():
    pass
"#,
        ),
        (
            "addons/plugin.gd",
            r#"extends Node
func _ready():
    pass
func unused_in_plugin():
    pass
"#,
        ),
    ]);
    let unused = find_unused_functions(&root, None, Some(&["addons".into()]));
    let names: Vec<_> = unused.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"unused_in_main"));
    assert!(!names.contains(&"unused_in_plugin"));
}

#[test]
fn default_is_test_path_outside_root_returns_false() {
    let (_dir, root) = project(&[]);
    let (_other, other_root) = project(&[("tests/foo.gd", "")]);
    assert!(!default_is_test_path(
        &root,
        &other_root.join("tests/foo.gd")
    ));
}

#[test]
fn default_is_test_path_stem_test_prefix_suffix() {
    let (_dir, root) = project(&[("test_something.gd", ""), ("something_test.gd", "")]);
    assert!(default_is_test_path(&root, &root.join("test_something.gd")));
    assert!(default_is_test_path(&root, &root.join("something_test.gd")));
}

#[test]
fn default_is_test_path_case_insensitive() {
    let (_dir, root) = project(&[
        ("Tests/foo.gd", ""),
        ("TEST/bar.gd", ""),
        ("Test_Something.gd", ""),
        ("something_Test.gd", ""),
    ]);
    assert!(default_is_test_path(&root, &root.join("Tests/foo.gd")));
    assert!(default_is_test_path(&root, &root.join("TEST/bar.gd")));
    assert!(default_is_test_path(&root, &root.join("Test_Something.gd")));
    assert!(default_is_test_path(&root, &root.join("something_Test.gd")));
}
