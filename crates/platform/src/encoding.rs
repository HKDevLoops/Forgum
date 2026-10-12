//! Universal UTF & Unicode Compatibility Engine.
//!
//! Enforces universal compatibility across all Unicode formats:
//! - UTF-8 with or without Byte Order Mark (BOM: `0xEF 0xBB 0xBF`).
//! - UTF-16 Little-Endian (`0xFF 0xFE`) and Big-Endian (`0xFE 0xFF`).
//! - UTF-32 Little-Endian (`0xFF 0xFE 0x00 0x00`) and Big-Endian (`0x00 0x00 0xFE 0xFF`).
//! - Transparent decoding and normalization to canonical UTF-8 strings.
//! - Security sanitization against Trojan Source BiDi override attacks (CVE-2021-42574)
//!   and terminal format disruption (null bytes, malformed surrogate pairs).
//!
//! Adheres strictly to workspace-wide `unsafe_code = "deny"`.

use std::fs;
use std::io;
use std::path::Path;

/// Decode raw bytes from any UTF format (UTF-8, UTF-16LE, UTF-16BE, UTF-32)
/// into a clean, normalized, and compatible UTF-8 String.
#[must_use]
#[allow(clippy::chunks_exact_to_as_chunks, clippy::manual_is_multiple_of)]
pub fn decode_text_compatible(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }

    // 1. Detect UTF-32 BOM (4 bytes)
    if bytes.len() >= 4 {
        if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
            // UTF-32 LE
            let words: Vec<u32> = bytes[4..]
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            return sanitize_and_normalize(
                &words
                    .into_iter()
                    .filter_map(char::from_u32)
                    .collect::<String>(),
            );
        } else if bytes.starts_with(&[0x00, 0x00, 0xFE, 0xFF]) {
            // UTF-32 BE
            let words: Vec<u32> = bytes[4..]
                .chunks_exact(4)
                .map(|c| u32::from_be_bytes([c[0], c[1], c[2], c[3]]))
                .collect();
            return sanitize_and_normalize(
                &words
                    .into_iter()
                    .filter_map(char::from_u32)
                    .collect::<String>(),
            );
        }
    }

    // 2. Detect UTF-8 BOM (3 bytes: 0xEF 0xBB 0xBF)
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        let text = String::from_utf8_lossy(&bytes[3..]);
        return sanitize_and_normalize(&text);
    }

    // 3. Detect UTF-16 BOM (2 bytes)
    if bytes.starts_with(&[0xFF, 0xFE]) {
        // UTF-16 LE
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        let decoded = char::decode_utf16(u16s)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>();
        return sanitize_and_normalize(&decoded);
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        // UTF-16 BE
        let u16s: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        let decoded = char::decode_utf16(u16s)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect::<String>();
        return sanitize_and_normalize(&decoded);
    }

    // 4. No BOM: Attempt clean UTF-8
    if let Ok(s) = std::str::from_utf8(bytes) {
        return sanitize_and_normalize(s);
    }

    // 5. Heuristic detection for BOM-less UTF-16 LE (frequent in Windows notepad/reg files)
    if bytes.len() >= 4 && bytes.len() % 2 == 0 {
        let null_ratio = bytes
            .iter()
            .enumerate()
            .filter(|(idx, &b)| idx % 2 == 1 && b == 0)
            .count() as f32
            / (bytes.len() / 2) as f32;
        if null_ratio > 0.6 {
            let u16s: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let decoded = char::decode_utf16(u16s)
                .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect::<String>();
            return sanitize_and_normalize(&decoded);
        }
    }

    // 6. Universal lossy fallback
    let s = String::from_utf8_lossy(bytes);
    sanitize_and_normalize(&s)
}

/// Read any text file regardless of UTF/Unicode encoding format.
/// Supports UTF-8, UTF-8-BOM, UTF-16LE, UTF-16BE, UTF-32, and legacy ASCII.
pub fn read_text_file_compatible(path: &Path) -> io::Result<String> {
    let raw_bytes = fs::read(path)?;
    Ok(decode_text_compatible(&raw_bytes))
}

/// Sanitize and normalize string for universal terminal compatibility:
/// - Normalizes line endings (`\r\n` and `\r` to `\n`).
/// - Strips zero-width BOM markers (`\u{feff}`) embedded in text.
/// - Strips Trojan Source BiDi override control characters (`U+202A..=U+202E`, `U+2066..=U+2069`).
/// - Strips null bytes (`\0`) to avoid Win32/C-string truncation.
#[must_use]
pub fn sanitize_and_normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_cr = false;

    for ch in s.chars() {
        // Strip BOM and null bytes
        if ch == '\u{feff}' || ch == '\0' {
            continue;
        }

        // Trojan Source BiDi overrides (CVE-2021-42574)
        if matches!(
            ch,
            '\u{202A}'
                | '\u{202B}'
                | '\u{202C}'
                | '\u{202D}'
                | '\u{202E}'
                | '\u{2066}'
                | '\u{2067}'
                | '\u{2068}'
                | '\u{2069}'
        ) {
            continue;
        }

        // Line ending normalization to standard LF
        if ch == '\r' {
            out.push('\n');
            prev_cr = true;
            continue;
        } else if ch == '\n' {
            if prev_cr {
                // Already emitted LF for preceding CR
                prev_cr = false;
                continue;
            }
            out.push('\n');
        } else {
            out.push(ch);
        }
        prev_cr = false;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8_plain() {
        let input = b"Hello world! Forgum mascot.";
        assert_eq!(decode_text_compatible(input), "Hello world! Forgum mascot.");
    }

    #[test]
    fn test_decode_utf8_with_bom() {
        let mut input = vec![0xEF, 0xBB, 0xBF];
        input.extend_from_slice(b"BOM prefixed mascot text.");
        assert_eq!(decode_text_compatible(&input), "BOM prefixed mascot text.");
    }

    #[test]
    fn test_decode_utf16_le_with_bom() {
        let text = "Hello from UTF-16 LE Forgum!";
        let mut input = vec![0xFF, 0xFE];
        for ch in text.encode_utf16() {
            input.extend_from_slice(&ch.to_le_bytes());
        }
        assert_eq!(decode_text_compatible(&input), text);
    }

    #[test]
    fn test_decode_utf16_be_with_bom() {
        let text = "Hello from UTF-16 BE Forgum!";
        let mut input = vec![0xFE, 0xFF];
        for ch in text.encode_utf16() {
            input.extend_from_slice(&ch.to_be_bytes());
        }
        assert_eq!(decode_text_compatible(&input), text);
    }

    #[test]
    fn test_decode_crlf_normalized() {
        let input = b"Line 1\r\nLine 2\rLine 3\nLine 4";
        assert_eq!(
            decode_text_compatible(input),
            "Line 1\nLine 2\nLine 3\nLine 4"
        );
    }

    #[test]
    fn test_bidi_override_sanitization() {
        // Embed LRO (U+202D) and RLO (U+202E)
        let malicious = "Normal \u{202E}Reversed\u{202C} End.";
        assert_eq!(sanitize_and_normalize(malicious), "Normal Reversed End.");
    }

    #[test]
    fn test_decode_utf32_le_with_bom() {
        let mut input = vec![0xFF, 0xFE, 0x00, 0x00];
        for ch in "Forgum UTF32".chars() {
            input.extend_from_slice(&(ch as u32).to_le_bytes());
        }
        assert_eq!(decode_text_compatible(&input), "Forgum UTF32");
    }

    #[test]
    fn test_decode_utf32_be_with_bom() {
        let mut input = vec![0x00, 0x00, 0xFE, 0xFF];
        for ch in "Forgum UTF32BE".chars() {
            input.extend_from_slice(&(ch as u32).to_be_bytes());
        }
        assert_eq!(decode_text_compatible(&input), "Forgum UTF32BE");
    }

    #[test]
    fn test_null_byte_sanitization() {
        let with_nulls = "Hello\0World\0Forgum";
        assert_eq!(sanitize_and_normalize(with_nulls), "HelloWorldForgum");
    }
}
