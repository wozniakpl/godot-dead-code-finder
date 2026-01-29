//! Extract function definitions from GDScript source.

use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::models::FunctionDef;

/// func name( ... ): or static func name( ... ):
/// Optional: -> Type at end. Name is identifier (letters, digits, underscore)
static FUNC_DEF_RE: OnceLock<Regex> = OnceLock::new();

fn func_def_re() -> &'static Regex {
    FUNC_DEF_RE.get_or_init(|| {
        Regex::new(
            r"(?m)^\s*(?:static\s+)?func\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)\s*(?:->[^:]+)?\s*:",
        )
        .unwrap()
    })
}

/// Replace string literal contents with spaces so we don't match inside strings.
pub fn strip_string_literals(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut i = 0;
    let n = source.len();
    let bytes = source.as_bytes();
    while i < n {
        let c = bytes[i] as char;
        if (c == '"' || c == '\'') && (i == 0 || bytes[i - 1] != b'\\') {
            let quote = c;
            result.push(quote);
            i += 1;
            if i < n {
                let next = bytes[i] as char;
                if next == quote {
                    result.push(quote);
                    i += 1;
                    continue;
                }
                if i + 2 <= n {
                    let two = std::str::from_utf8(&bytes[i..i + 2]).unwrap_or("");
                    if two == format!("{}{}", quote, quote) {
                        let triple = format!("{}{}{}", quote, quote, quote);
                        let rest = std::str::from_utf8(&bytes[i + 3..]).unwrap_or("");
                        if let Some(pos) = rest.find(&triple) {
                            let end = i + 3 + pos + 3;
                            result.extend(std::iter::repeat_n(' ', end - i));
                            i = end;
                            continue;
                        } else {
                            result.extend(std::iter::repeat_n(' ', n - i));
                            break;
                        }
                    }
                }
            }
            while i < n {
                if bytes[i] == b'\\' && i + 1 < n {
                    result.push_str("  ");
                    i += 2;
                    continue;
                }
                if (bytes[i] as char) == quote {
                    result.push(quote);
                    i += 1;
                    break;
                }
                result.push(' ');
                i += 1;
            }
            continue;
        }
        result.push(c);
        i += 1;
    }
    result
}

/// Extract all function definitions from a GDScript source (top-level and inner classes).
pub fn find_function_definitions(path: &Path, source: &str) -> Vec<FunctionDef> {
    let mut out = Vec::new();
    for cap in func_def_re().captures_iter(source) {
        let m = cap.get(0).unwrap();
        let name_match = cap.get(1).unwrap();
        let line_no = (source[..name_match.start()].matches('\n').count() + 1) as u32;
        let name = name_match.as_str().to_string();
        let full = m.as_str();
        let is_static = full.contains("static");
        out.push(FunctionDef {
            name,
            file: path.to_path_buf(),
            line: line_no,
            is_static,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_string_literals_triple_quote() {
        let s = r#"x """ not a call() here """ y"#;
        let out = strip_string_literals(s);
        assert!(out.starts_with("x "));
        assert!(out.ends_with(" y"));
        assert!(!out.contains("call") || out.matches("call").count() == 0);
    }

    #[test]
    fn strip_string_literals_escape() {
        let s = r#""a\\b""#;
        let out = strip_string_literals(s);
        assert_eq!(out.len(), s.len());
    }
}
