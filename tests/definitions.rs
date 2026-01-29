//! Integration tests for find_function_definitions.

use std::path::Path;

use gdcf::scanner::find_function_definitions;

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
