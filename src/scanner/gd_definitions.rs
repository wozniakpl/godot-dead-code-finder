//! Extract function definitions from GDScript source.

use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::models::FunctionDef;

/// func name( ... ): or static func name( ... ):
/// Optional: -> Type at end. Name is identifier (letters, digits, underscore)
static FUNC_DEF_RE: OnceLock<Regex> = OnceLock::new();

/// Regex: # then optional space then gdcf-ignore | dead-code-ignore | TODO: dead-code (case-insensitive).
static IGNORE_DEAD_CODE_RE: OnceLock<Regex> = OnceLock::new();

fn func_def_re() -> &'static Regex {
    FUNC_DEF_RE.get_or_init(|| {
        Regex::new(
            r"(?m)^\s*(?:static\s+)?func\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)\s*(?:->[^:]+)?\s*:",
        )
        .unwrap()
    })
}

fn ignore_dead_code_re() -> &'static Regex {
    IGNORE_DEAD_CODE_RE.get_or_init(|| {
        Regex::new(r"(?i)#\s*(?:gdcf-ignore|dead-code-ignore|TODO:\s*dead-code)").unwrap()
    })
}

/// True if line contains an ignore-dead-code marker after a #.
fn line_has_ignore_marker(line: &str) -> bool {
    ignore_dead_code_re().is_match(line)
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
/// Functions tagged with `# gdcf-ignore`, `# dead-code-ignore`, or `# TODO: dead-code`
/// (on the same line after `:` or on the next line) get `ignore_dead_code: true`.
pub fn find_function_definitions(path: &Path, source: &str) -> Vec<FunctionDef> {
    let mut out = Vec::new();
    for cap in func_def_re().captures_iter(source) {
        let m = cap.get(0).unwrap();
        let name_match = cap.get(1).unwrap();
        let line_no = (source[..name_match.start()].matches('\n').count() + 1) as u32;
        let name = name_match.as_str().to_string();
        let full = m.as_str();
        let is_static = full.contains("static");

        // Same line: from end of match to end of line
        let rest_start = m.end();
        let same_line_end = source[rest_start..]
            .find('\n')
            .map(|o| rest_start + o)
            .unwrap_or(source.len());
        let same_line = &source[rest_start..same_line_end];

        // Next line (first line of body)
        let next_line = if same_line_end < source.len() {
            let next_start = same_line_end + 1;
            let next_end = source[next_start..]
                .find('\n')
                .map(|o| next_start + o)
                .unwrap_or(source.len());
            &source[next_start..next_end]
        } else {
            ""
        };

        let ignore_dead_code =
            line_has_ignore_marker(same_line) || line_has_ignore_marker(next_line);

        out.push(FunctionDef {
            name,
            file: path.to_path_buf(),
            line: line_no,
            is_static,
            ignore_dead_code,
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

    #[test]
    fn find_function_definitions_ignore_dead_code_same_line() {
        let source = "func kept_for_later(): # gdcf-ignore\n    pass";
        let defs = find_function_definitions(Path::new("a.gd"), source);
        assert_eq!(defs.len(), 1);
        assert!(defs[0].ignore_dead_code);
    }

    #[test]
    fn find_function_definitions_ignore_dead_code_next_line() {
        let source = "func kept_for_later():\n    # TODO: dead-code\n    pass";
        let defs = find_function_definitions(Path::new("a.gd"), source);
        assert_eq!(defs.len(), 1);
        assert!(defs[0].ignore_dead_code);
    }

    #[test]
    fn find_function_definitions_no_ignore_by_default() {
        let source = "func foo():\n    pass";
        let defs = find_function_definitions(Path::new("a.gd"), source);
        assert_eq!(defs.len(), 1);
        assert!(!defs[0].ignore_dead_code);
    }
}
