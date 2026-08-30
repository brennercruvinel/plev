//! Extract `plev_narrate! { ... }` blocks from Rust source text.
//!
//! Used by the hot-reload file watcher to find narrate macro invocations
//! in `.rs` files, extract their DSL bodies, and report the line number
//! of each invocation for override-map keying.

/// Extract all `plev_narrate! { ... }` blocks from Rust source text.
///
/// Returns `(line_number, block_content)` pairs. Handles nested braces,
/// string literals, and line comments within the macro body.
pub fn extract_narrate_blocks(source: &str) -> Vec<(u32, String)> {
    let mut results = Vec::new();
    let bytes = source.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if let Some(after_bang) = match_macro_name(source, i) {
            let macro_start = i;
            i = after_bang;

            // skip whitespace
            while i < len && bytes[i].is_ascii_whitespace() {
                i += 1;
            }

            if i < len && bytes[i] == b'{' {
                let line = u32::try_from(
                    source[..macro_start]
                        .bytes()
                        .filter(|&b| b == b'\n')
                        .count(),
                )
                .expect("source line count exceeds u32")
                    + 1;
                i += 1; // consume opening {
                let content_start = i;
                let mut depth = 1;

                while i < len && depth > 0 {
                    match bytes[i] {
                        b'{' => depth += 1,
                        b'}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        b'"' => {
                            i += 1;
                            while i < len && bytes[i] != b'"' {
                                if bytes[i] == b'\\' {
                                    i += 1;
                                }
                                i += 1;
                            }
                        }
                        b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                            while i < len && bytes[i] != b'\n' {
                                i += 1;
                            }
                            continue; // don't double-increment
                        }
                        _ => {}
                    }
                    i += 1;
                }

                if depth == 0 {
                    let content = &source[content_start..i];
                    results.push((line, content.to_string()));
                    i += 1; // skip closing }
                }
            }
        } else {
            i += 1;
        }
    }

    results
}

fn match_macro_name(source: &str, pos: usize) -> Option<usize> {
    let rest = &source[pos..];
    for pattern in &["plev_narrate!", "plev_narrate!"] {
        if rest.starts_with(pattern) {
            if pos > 0 {
                let prev = source.as_bytes()[pos - 1];
                if prev.is_ascii_alphanumeric() || prev == b'_' {
                    continue;
                }
            }
            return Some(pos + pattern.len());
        }
    }
    None
}
