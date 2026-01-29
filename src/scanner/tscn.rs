//! Find signal connection references in .tscn files.

use std::path::Path;

use regex::Regex;

/// Godot .tscn [connection ... method="callback_name"] – method= is the script callback
static TSCN_METHOD_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();

fn tscn_method_re() -> &'static Regex {
    TSCN_METHOD_RE
        .get_or_init(|| Regex::new(r#"method\s*=\s*["']([a-zA-Z_][a-zA-Z0-9_]*)["']"#).unwrap())
}

/// Find signal connection method names in a .tscn file.
/// Returns list of (function_name, line_number) for each method="..." in [connection] blocks.
pub fn find_tscn_references(_path: &Path, source: &str) -> Vec<(String, u32)> {
    let mut refs = Vec::new();
    let line_at = |pos: usize| -> u32 { (source[..pos].matches('\n').count() + 1) as u32 };
    for cap in tscn_method_re().captures_iter(source) {
        let m = cap.get(1).unwrap();
        refs.push((m.as_str().to_string(), line_at(m.start())));
    }
    refs
}
