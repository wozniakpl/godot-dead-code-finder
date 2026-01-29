//! Find function references in GDScript source.

use std::path::Path;

use regex::Regex;

use super::gd_definitions::strip_string_literals;

/// Find references to functions in source (calls, connect(callback), call("name"), etc.).
/// Returns list of (function_name, line_number).
/// Source should be normalized (see util.normalize_source) for consistent line endings.
pub fn find_function_references(_path: &Path, source: &str) -> Vec<(String, u32)> {
    let stripped = strip_string_literals(source);
    let mut refs: Vec<(String, u32)> = Vec::new();

    let line_at = |pos: usize| -> u32 { (source[..pos].matches('\n').count() + 1) as u32 };

    // 1. call("func_name") or call_deferred("func_name") – string is the name (use source)
    // (no lookbehind in Rust regex: use (?:^|\n|[^...]) prefix)
    let call_patterns = [
        (r#"\.call\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#, 1),
        (
            r#"(?:^|\n|[^a-zA-Z0-9.])call\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#,
            1,
        ),
        (
            r#"call_deferred\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#,
            1,
        ),
        (
            r#"\.call_deferred\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#,
            1,
        ),
    ];
    for (pat, group_idx) in call_patterns {
        let re = Regex::new(pat).unwrap();
        for cap in re.captures_iter(source) {
            let m = cap.get(group_idx).unwrap();
            refs.push((m.as_str().to_string(), line_at(m.start())));
        }
    }

    // 1b. Callable(self, "func_name") or Callable(obj, "func_name")
    let callable_re = Regex::new(
        r#"Callable\s*\(\s*(?:self|[a-zA-Z_][a-zA-Z0-9_.]*)\s*,\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#,
    )
    .unwrap();
    for cap in callable_re.captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }

    // 2. .connect(some_func) or .connect(self.some_func)
    let connect_re = Regex::new(r"\.connect\s*\(\s*(?:self\.)?([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    for cap in connect_re.captures_iter(&stripped) {
        let m = cap.get(0).unwrap();
        refs.push((cap.get(1).unwrap().as_str().to_string(), line_at(m.start())));
    }

    // 2b. obj.method_name( – explicit method call
    let method_call_re = Regex::new(r"\.\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
    for cap in method_call_re.captures_iter(&stripped) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }
    for cap in method_call_re.captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }

    // 2c. obj["method_name"]( or obj['method_name']( – dynamic method call
    let bracket_call_re = Regex::new(r#"\[\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']\s*\]\s*\("#).unwrap();
    for cap in bracket_call_re.captures_iter(&stripped) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }

    // 3. identifier( – direct call (not after a dot); (?:^|\n|[^...]) replaces lookbehind
    let id_call_re = Regex::new(r"(?:^|\n|[^a-zA-Z0-9_.])([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap();
    let keywords: std::collections::HashSet<&str> = [
        "if", "elif", "else", "for", "while", "match", "when", "return", "pass", "break",
        "continue", "and", "or", "not", "in", "is", "as", "await", "func", "class", "static",
        "const", "var", "signal", "extends", "super", "true", "false", "null", "self", "print",
        "assert", "preload",
    ]
    .into_iter()
    .collect();

    let mut add_id_calls = |text: &str| {
        for cap in id_call_re.captures_iter(text) {
            let name = cap.get(1).unwrap().as_str();
            if keywords.contains(name) {
                continue;
            }
            refs.push((name.to_string(), line_at(cap.get(1).unwrap().start())));
        }
    };
    add_id_calls(&stripped);
    add_id_calls(source);

    refs
}
