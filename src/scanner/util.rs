//! Shared utilities for source normalization.

/// Normalize line endings and BOM so regex and line counts are consistent across platforms.
pub fn normalize_source(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    let mut s = text.replace("\r\n", "\n").replace('\r', "\n");
    if s.starts_with('\u{feff}') {
        s = s.chars().skip(1).collect();
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn normalize_source_crlf() {
        assert!(!normalize_source("a\r\nb\r\nc").contains('\r'));
        assert_eq!(normalize_source("a\r\nb"), "a\nb");
    }
}
