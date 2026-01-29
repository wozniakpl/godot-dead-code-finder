//! Integration tests for normalize_source and strip_string_literals.

use gdcf::scanner::gd_definitions::strip_string_literals;
use gdcf::scanner::util::normalize_source;

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
