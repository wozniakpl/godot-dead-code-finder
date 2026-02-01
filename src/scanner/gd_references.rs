//! Find function references in GDScript source.

use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::gd_definitions::strip_string_literals;

static CALL_DOT_RE: OnceLock<Regex> = OnceLock::new();
static CALL_STANDALONE_RE: OnceLock<Regex> = OnceLock::new();
static CALL_DEFERRED_RE: OnceLock<Regex> = OnceLock::new();
static CALL_DOT_DEFERRED_RE: OnceLock<Regex> = OnceLock::new();
static CALLABLE_RE: OnceLock<Regex> = OnceLock::new();
static CONNECT_RE: OnceLock<Regex> = OnceLock::new();
static METHOD_CALL_RE: OnceLock<Regex> = OnceLock::new();
static BRACKET_CALL_RE: OnceLock<Regex> = OnceLock::new();
static ID_CALL_RE: OnceLock<Regex> = OnceLock::new();
static NESTED_CALL_RE: OnceLock<Regex> = OnceLock::new();
static ASSIGN_RHS_RE: OnceLock<Regex> = OnceLock::new();
/// Identifier as first argument of a call: foo(callback, ...) or foo(callback) — e.g. tween_method(set_master_volume, ...)
static FIRST_ARG_IDENT_RE: OnceLock<Regex> = OnceLock::new();

static KEYWORDS: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();

fn keywords() -> &'static std::collections::HashSet<&'static str> {
    KEYWORDS.get_or_init(|| {
        [
            "if", "elif", "else", "for", "while", "match", "when", "return", "pass", "break",
            "continue", "and", "or", "not", "in", "is", "as", "await", "func", "class", "static",
            "const", "var", "signal", "extends", "super", "true", "false", "null", "self", "print",
            "assert", "preload",
        ]
        .into_iter()
        .collect()
    })
}

/// Find references to functions in source (calls, connect(callback), call("name"), etc.).
/// Returns list of (function_name, line_number).
/// Source should be normalized (see util.normalize_source) for consistent line endings.
pub fn find_function_references(_path: &Path, source: &str) -> Vec<(String, u32)> {
    let stripped = strip_string_literals(source);
    let mut refs: Vec<(String, u32)> = Vec::new();
    let line_at = |pos: usize| -> u32 { (source[..pos].matches('\n').count() + 1) as u32 };

    // 1. call("func_name") or call_deferred("func_name") – string is the name (use source)
    let re = CALL_DOT_RE
        .get_or_init(|| Regex::new(r#"\.call\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#).unwrap());
    for cap in re.captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }
    let re = CALL_STANDALONE_RE.get_or_init(|| {
        Regex::new(r#"(?:^|\n|[^a-zA-Z0-9.])call\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#).unwrap()
    });
    for cap in re.captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }
    let re = CALL_DEFERRED_RE.get_or_init(|| {
        Regex::new(r#"call_deferred\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#).unwrap()
    });
    for cap in re.captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }
    let re = CALL_DOT_DEFERRED_RE.get_or_init(|| {
        Regex::new(r#"\.call_deferred\s*\(\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#).unwrap()
    });
    for cap in re.captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }

    // 1b. Callable(self, "func_name") or Callable(obj, "func_name")
    let re = CALLABLE_RE.get_or_init(|| {
        Regex::new(
            r#"Callable\s*\(\s*(?:self|[a-zA-Z_][a-zA-Z0-9_.]*)\s*,\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#,
        )
        .unwrap()
    });
    for cap in re.captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }

    // 2. .connect(some_func) or .connect(self.some_func)
    let re = CONNECT_RE.get_or_init(|| {
        Regex::new(r"\.connect\s*\(\s*(?:self\.)?([a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
    });
    for cap in re.captures_iter(&stripped) {
        let m = cap.get(0).unwrap();
        refs.push((cap.get(1).unwrap().as_str().to_string(), line_at(m.start())));
    }

    // 2b. obj.method_name( – explicit method call (stripped only to avoid refs inside strings)
    let re =
        METHOD_CALL_RE.get_or_init(|| Regex::new(r"\.\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap());
    for cap in re.captures_iter(&stripped) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }

    // 2c. obj["method_name"]( or obj['method_name']( – dynamic method call
    let re = BRACKET_CALL_RE
        .get_or_init(|| Regex::new(r#"\[\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']\s*\]\s*\("#).unwrap());
    for cap in re.captures_iter(&stripped) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }

    // 3. identifier( – direct call; 3b. ( identifier ( – nested call (stripped only to avoid refs inside strings)
    let id_re = ID_CALL_RE.get_or_init(|| {
        Regex::new(r"(?:^|\n|[^a-zA-Z0-9_.])([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap()
    });
    let nested_re =
        NESTED_CALL_RE.get_or_init(|| Regex::new(r"\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\(").unwrap());
    let kw = keywords();
    for cap in id_re.captures_iter(&stripped) {
        let name = cap.get(1).unwrap().as_str();
        if !kw.contains(name) {
            refs.push((name.to_string(), line_at(cap.get(1).unwrap().start())));
        }
    }
    for cap in nested_re.captures_iter(&stripped) {
        let name = cap.get(1).unwrap().as_str();
        if !kw.contains(name) {
            refs.push((name.to_string(), line_at(cap.get(1).unwrap().start())));
        }
    }

    // 4. = func_name (function used as value)
    let re = ASSIGN_RHS_RE
        .get_or_init(|| Regex::new(r"=\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*([;\n,\)\]\}\(]?)").unwrap());
    for cap in re.captures_iter(&stripped) {
        let name = cap.get(1).unwrap().as_str();
        let next = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        if next == "(" || kw.contains(name) {
            continue;
        }
        refs.push((name.to_string(), line_at(cap.get(1).unwrap().start())));
    }

    // 5. identifier as first argument of a call: tween_method(set_master_volume, from, to, duration)
    let re = FIRST_ARG_IDENT_RE
        .get_or_init(|| Regex::new(r"\(\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*[,\)]").unwrap());
    for cap in re.captures_iter(&stripped) {
        let name = cap.get(1).unwrap().as_str();
        if !kw.contains(name) {
            refs.push((name.to_string(), line_at(cap.get(1).unwrap().start())));
        }
    }

    refs
}
