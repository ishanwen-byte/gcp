//! Enhanced JSON parsing utilities for GitHub API responses
//! Minimal implementation without external dependencies

/// Extract a required JSON string field
pub fn extract_json_field(json_str: &str, field: &str) -> String {
    let pattern = format!("\"{}\":", field);

    // Find the field position
    let mut start = match json_str.find(&pattern) {
        Some(pos) => pos + pattern.len(),
        None => return String::new(),
    };

    // Skip whitespace after the colon
    let after_field = json_str[start..].trim_start();
    start = json_str.len() - after_field.len();

    // Check if it's null
    if after_field.starts_with("null") {
        return String::new();
    }

    // Extract quoted string value
    if let Some(quote_start) = after_field.find('"') {
        let value_start = start + quote_start + 1;
        let remaining = &json_str[value_start..];

        // Find the closing quote, handling escaped quotes
        let mut escaped = false;
        let mut quote_end = None;

        for (i, c) in remaining.chars().enumerate() {
            if escaped {
                escaped = false;
                continue;
            }

            match c {
                '\\' => escaped = true,
                '"' => {
                    quote_end = Some(i);
                    break;
                }
                _ => {}
            }
        }

        if let Some(end) = quote_end {
            return unescape_json_string(&remaining[..end]);
        }
    }

    String::new()
}

/// Unescape a raw JSON string body (content between quotes) into its real value.
/// Handles \", \\, \/, \b, \f, \n, \r, \t and \uXXXX escapes.
fn unescape_json_string(raw: &str) -> String {
    if !raw.contains('\\') {
        return raw.to_string();
    }

    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }

        match chars.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some('/') => out.push('/'),
            Some('b') => out.push('\u{0008}'),
            Some('f') => out.push('\u{000C}'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let mut code = 0u32;
                let mut valid = true;
                for _ in 0..4 {
                    match chars.next() {
                        Some(h) => {
                            code = code.wrapping_shl(4)
                                | h.to_digit(16).unwrap_or_else(|| {
                                    valid = false;
                                    0
                                });
                        }
                        None => {
                            valid = false;
                            break;
                        }
                    }
                }
                if valid {
                    // Handle surrogate pairs for full Unicode support
                    if (0xD800..0xDC00).contains(&code) {
                        // High surrogate: try to read the low surrogate
                        let mut lookahead = chars.clone();
                        if let (Some('\\'), Some('u')) = (lookahead.next(), lookahead.next()) {
                            let mut low = 0u32;
                            let mut low_valid = true;
                            for _ in 0..4 {
                                match lookahead.next() {
                                    Some(h) => {
                                        low = low.wrapping_shl(4)
                                            | h.to_digit(16).unwrap_or_else(|| {
                                                low_valid = false;
                                                0
                                            });
                                    }
                                    None => {
                                        low_valid = false;
                                        break;
                                    }
                                }
                            }
                            if low_valid && (0xDC00..0xE000).contains(&low) {
                                chars = lookahead;
                                let combined = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                                if let Some(ch) = char::from_u32(combined) {
                                    out.push(ch);
                                    continue;
                                }
                            }
                        }
                        // Invalid surrogate: push replacement char
                        out.push('\u{FFFD}');
                    } else if let Some(ch) = char::from_u32(code) {
                        out.push(ch);
                    } else {
                        out.push('\u{FFFD}');
                    }
                } else {
                    out.push('\u{FFFD}');
                }
            }
            _ => out.push('\u{FFFD}'),
        }
    }

    out
}

/// Extract an optional JSON string field
pub fn extract_optional_json_field(json_str: &str, field: &str) -> Option<String> {
    let value = extract_json_field(json_str, field);
    if value.is_empty() {
        // Check if it was explicitly null vs empty string
        let pattern = format!("\"{}\":", field);
        if let Some(pos) = json_str.find(&pattern) {
            let after_field = &json_str[pos + pattern.len()..];
            let after_field = after_field.trim_start();
            if after_field.starts_with("null") {
                return None;
            }
        }
        Some(value)
    } else {
        Some(value)
    }
}

/// Parse GitHub API file info from JSON
pub struct GitHubFile {
    pub name: String,
    pub path: String,
    pub file_type: String,
    pub download_url: Option<String>,
    pub content: Option<String>,
    pub encoding: Option<String>,
}

impl GitHubFile {
    pub fn from_json(json_str: &str) -> Result<Self, crate::error::GcpError> {
        let name = extract_json_field(json_str, "name");
        let path = extract_json_field(json_str, "path");
        let file_type = extract_json_field(json_str, "type");

        if name.is_empty() {
            return Err(crate::error::GcpError::ParseError(
                "Invalid JSON format: missing or empty 'name' field".to_string(),
            ));
        }

        let download_url =
            extract_optional_json_field(json_str, "download_url").filter(|url| !url.is_empty());

        let content = extract_optional_json_field(json_str, "content").filter(|c| !c.is_empty());

        let encoding = extract_optional_json_field(json_str, "encoding").filter(|e| !e.is_empty());

        Ok(GitHubFile {
            name,
            path,
            file_type,
            download_url,
            content,
            encoding,
        })
    }
}

/// Parse JSON array of GitHub files
pub fn parse_github_file_array(json_str: &str) -> Result<Vec<GitHubFile>, crate::error::GcpError> {
    let json_str = json_str.trim();

    if !json_str.starts_with('[') {
        return Err(crate::error::GcpError::ParseError(
            "Expected JSON array".to_string(),
        ));
    }

    let array_content = &json_str[1..json_str.len().saturating_sub(1)];
    let mut files = Vec::new();

    // Parse individual JSON objects in the array
    let mut current_object = String::new();
    let mut brace_count = 0;
    let mut in_string = false;
    let mut escaped = false;

    for ch in array_content.chars() {
        if in_string {
            if escaped {
                escaped = false;
                current_object.push(ch);
                continue;
            }

            match ch {
                '\\' => {
                    escaped = true;
                    current_object.push(ch);
                }
                '"' => {
                    in_string = false;
                    current_object.push(ch);
                }
                _ => {
                    current_object.push(ch);
                }
            }
        } else {
            match ch {
                '"' => {
                    in_string = true;
                    current_object.push(ch);
                }
                '{' => {
                    if brace_count == 0 {
                        current_object.clear();
                    }
                    brace_count += 1;
                    current_object.push(ch);
                }
                '}' => {
                    brace_count -= 1;
                    current_object.push(ch);

                    if brace_count == 0 {
                        if let Ok(file) = GitHubFile::from_json(&current_object) {
                            files.push(file);
                        }
                        current_object.clear();
                    }
                }
                ',' if brace_count == 0 => {
                    // Skip comma between objects
                    continue;
                }
                ' ' | '\t' | '\n' | '\r' if brace_count == 0 => {
                    // Skip whitespace between objects
                    continue;
                }
                _ => {
                    if brace_count > 0 {
                        current_object.push(ch);
                    }
                }
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_field() {
        let json = r#"{"name": "test.txt", "size": 100}"#;
        assert_eq!(extract_json_field(json, "name"), "test.txt");
    }

    #[test]
    fn test_extract_field_with_spaces() {
        let json = r#"{"name":  "test.txt"  }"#;
        assert_eq!(extract_json_field(json, "name"), "test.txt");
    }

    #[test]
    fn test_extract_field_with_escaped_quotes() {
        let json = r#"{"content": "line1\nline2\"quote\""}"#;
        assert_eq!(extract_json_field(json, "content"), "line1\nline2\"quote\"");
    }

    #[test]
    fn test_extract_field_with_unicode_escape() {
        let json = r#"{"name": "\u4f60\u597d"}"#;
        assert_eq!(extract_json_field(json, "name"), "你好");
    }

    #[test]
    fn test_extract_field_with_surrogate_pair() {
        let json = r#"{"name": "\ud83d\ude00"}"#;
        assert_eq!(extract_json_field(json, "name"), "😀");
    }

    #[test]
    fn test_extract_field_backslash_and_tab() {
        let json = r#"{"path": "a\\b\tc"}"#;
        assert_eq!(extract_json_field(json, "path"), "a\\b\tc");
    }

    #[test]
    fn test_extract_null_field() {
        let json = r#"{"download_url": null}"#;
        assert_eq!(extract_json_field(json, "download_url"), "");
        assert_eq!(extract_optional_json_field(json, "download_url"), None);
    }

    #[test]
    fn test_parse_github_file() {
        let json = r#"{
            "name": "test.txt",
            "path": "src/test.txt",
            "type": "file",
            "download_url": "https://example.com/test.txt",
            "content": "SGVsbG8=",
            "encoding": "base64"
        }"#;

        let file = GitHubFile::from_json(json).unwrap();
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.path, "src/test.txt");
        assert_eq!(file.file_type, "file");
        assert_eq!(
            file.download_url,
            Some("https://example.com/test.txt".to_string())
        );
        assert_eq!(file.content, Some("SGVsbG8=".to_string()));
        assert_eq!(file.encoding, Some("base64".to_string()));
    }

    #[test]
    fn test_parse_file_array() {
        let json = r#"[
            {"name": "file1.txt", "path": "file1.txt", "type": "file"},
            {"name": "file2.txt", "path": "file2.txt", "type": "file"}
        ]"#;

        let files = parse_github_file_array(json).unwrap();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].name, "file1.txt");
        assert_eq!(files[1].name, "file2.txt");
    }
}
