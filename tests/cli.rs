//! Integration tests for CLI (run, Args) – exercise gdcf::cli for coverage.

mod common;
use common::project;

use clap::Parser;
use gdcf::cli::{run, Args};

fn run_cli(args: &[&str]) -> i32 {
    let argv: Vec<&str> = std::iter::once("godot-dead-code")
        .chain(args.iter().copied())
        .collect();
    run(Args::parse_from(argv))
}

#[test]
fn cli_empty_project_exit_zero() {
    let (_dir, root) = project(&[("main.gd", "extends Node\nfunc _ready():\n    pass\n")]);
    let code = run_cli(&[root.to_str().unwrap()]);
    assert_eq!(code, 0);
}

#[test]
fn cli_unused_function_exit_one() {
    let (_dir, root) = project(&[(
        "main.gd",
        r#"extends Node
func _ready():
    pass
func never_called():
    pass
"#,
    )]);
    let code = run_cli(&[root.to_str().unwrap()]);
    assert_eq!(code, 1);
}

#[test]
fn cli_quiet_no_unused_exit_zero() {
    let (_dir, root) = project(&[("main.gd", "extends Node\nfunc _ready():\n    pass\n")]);
    let code = run_cli(&["-q", root.to_str().unwrap()]);
    assert_eq!(code, 0);
}

#[test]
fn cli_quiet_with_unused_exit_one() {
    let (_dir, root) = project(&[(
        "main.gd",
        r#"extends Node
func _ready():
    pass
func unused():
    pass
"#,
    )]);
    let code = run_cli(&["--quiet", root.to_str().unwrap()]);
    assert_eq!(code, 1);
}

#[test]
fn cli_verbose_summary() {
    let (_dir, root) = project(&[("main.gd", "extends Node\nfunc _ready():\n    pass\n")]);
    let code = run_cli(&["-v", root.to_str().unwrap()]);
    assert_eq!(code, 0);
}

#[test]
fn cli_verbose_more_paths() {
    let (_dir, root) = project(&[("main.gd", "extends Node\nfunc _ready():\n    pass\n")]);
    let code = run_cli(&["-vv", root.to_str().unwrap()]);
    assert_eq!(code, 0);
}

#[test]
fn cli_debug_function_exit_zero() {
    let (_dir, root) = project(&[(
        "main.gd",
        r#"extends Node
func _ready():
    pass
func foo():
    pass
"#,
    )]);
    let code = run_cli(&["--debug-function", "foo", root.to_str().unwrap()]);
    assert_eq!(code, 0);
}

#[test]
fn cli_exclude_dir() {
    let (_dir, root) = project(&[
        ("main.gd", "extends Node\nfunc _ready(): pass\n"),
        (
            "addons/plugin.gd",
            "extends Node\nfunc _ready(): pass\nfunc only_in_plugin(): pass\n",
        ),
    ]);
    let code = run_cli(&[root.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "addons excluded by default so only_in_plugin not reported"
    );
}

#[test]
fn cli_exclude_dir_override() {
    let (_dir, root) = project(&[
        (
            "main.gd",
            "extends Node\nfunc _ready(): pass\nfunc unused_main(): pass\n",
        ),
        ("other/script.gd", "extends Node\nfunc _ready(): pass\n"),
    ]);
    let code = run_cli(&["--exclude-dir", "other", root.to_str().unwrap()]);
    assert_eq!(code, 1, "unused_main in main.gd reported");
}

#[test]
fn cli_no_default_excludes() {
    let (_dir, root) = project(&[
        ("main.gd", "extends Node\nfunc _ready(): pass\n"),
        (
            "addons/plugin.gd",
            "extends Node\nfunc _ready(): pass\nfunc only_in_plugin(): pass\n",
        ),
    ]);
    let code = run_cli(&["--no-default-excludes", root.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "with --no-default-excludes, addons is scanned so only_in_plugin is reported as unused"
    );
}

#[test]
fn cli_tests_dir_only_test_referenced() {
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
    let code = run_cli(&[root.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "only_called_from_test only referenced from test code"
    );
}
