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
/// - Strips all ANSI escape sequences (CSI, OSC, DCS, APC, PM, SOS, DEC modes).
/// - Strips all non-printable C0 and C1 control characters (BEL, BS, DEL, etc.), preserving tabs and newlines.
/// - Normalizes line endings (`\r\n` and `\r` to `\n`).
/// - Strips zero-width BOM markers (`\u{feff}`) embedded in text.
/// - Strips Trojan Source BiDi override control characters (`U+202A..=U+202E`, `U+2066..=U+2069`).
/// - Strips null bytes (`\0`) to avoid Win32/C-string truncation.
#[must_use]
pub fn sanitize_and_normalize(s: &str) -> String {
    strip_ansi_and_controls(s)
}

/// Strip all ANSI escape sequences, terminal controls, and dangerous C0/C1 control codes
/// from `s`, preserving standard printable Unicode text, spaces, tabs (`\t`), and line breaks (`\n`).
///
/// Strips:
/// - CSI sequences: `\x1b[ ... [0x40..=0x7E]` and C1 `\u{009B} ... [0x40..=0x7E]`
/// - OSC sequences: `\x1b] ... (\x07 | \x1b\\ | \u{009C})` and C1 `\u{009D} ... (\x07 | \x1b\\ | \u{009C})`
/// - DCS / APC / PM / SOS: `\x1b[P_X^]` and C1 `\u{0090}`, `\u{0098}`, `\u{009E}`, `\u{009F}`
/// - Two-byte escape sequences: `\x1b[0x20..=0x7E]` (e.g. `\x1bc`, `\x1b=`, `\x1b7`, `\x1b8`, `\x1bM`)
/// - C0 control characters (`\x00..=\x08`, `\x0B`, `\x0C`, `\x0E..=\x1F`, `\x7F`)
/// - C1 control characters (`0x80..=0x9F`)
/// - Trojan Source BiDi overrides (`U+202A..=U+202E`, `U+2066..=U+2069`)
/// - Null bytes and BOM markers
#[must_use]
pub fn strip_ansi_and_controls(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut prev_cr = false;

    while let Some(ch) = chars.next() {
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
                prev_cr = false;
                continue;
            }
            out.push('\n');
            prev_cr = false;
            continue;
        }
        prev_cr = false;

        // Escape sequences starting with ESC (\x1b)
        if ch == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next(); // consume '['
                    for next in chars.by_ref() {
                        let b = next as u32;
                        if (0x40..=0x7E).contains(&b) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next(); // consume ']'
                    while let Some(next) = chars.next() {
                        if next == '\x07' || next == '\u{009C}' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next(); // consume '\\'
                            break;
                        }
                    }
                }
                Some('P') | Some('_') | Some('^') | Some('X') => {
                    chars.next(); // consume prefix
                    while let Some(next) = chars.next() {
                        if next == '\x07' || next == '\u{009C}' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some(&next) if (0x20..=0x7E).contains(&(next as u32)) => {
                    chars.next(); // consume 2-byte escape
                }
                _ => {}
            }
            continue;
        }

        // C1 control characters (U+0080..=U+009F)
        let code = ch as u32;
        if (0x80..=0x9F).contains(&code) {
            match ch {
                '\u{009B}' => {
                    // C1 CSI
                    for next in chars.by_ref() {
                        let b = next as u32;
                        if (0x40..=0x7E).contains(&b) {
                            break;
                        }
                    }
                }
                '\u{009D}' => {
                    // C1 OSC
                    while let Some(next) = chars.next() {
                        if next == '\x07' || next == '\u{009C}' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                '\u{0090}' | '\u{0098}' | '\u{009E}' | '\u{009F}' => {
                    // C1 DCS / SOS / PM / APC
                    while let Some(next) = chars.next() {
                        if next == '\x07' || next == '\u{009C}' {
                            break;
                        }
                        if next == '\x1b' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        // C0 control characters (0x00..0x1F except \n, \t; and 0x7F DEL)
        if ch == '\t' {
            out.push('\t');
        } else if code >= 0x20 && code != 0x7F {
            out.push(ch);
        }
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

    #[test]
    fn test_strip_ansi_csi_and_osc() {
        let dirty = "\x1b[31;1mRed Alert\x1b[0m \x1b]52;c;evil_clipboard\x07Clean Text";
        assert_eq!(strip_ansi_and_controls(dirty), "Red Alert Clean Text");
    }

    #[test]
    fn test_strip_dec_modes_and_cursor_hijack() {
        let malicious = "\x1b[?1049h\x1b[?25l\x1b[2J\x1b[10;20HMascot Art\x1b[6n";
        assert_eq!(strip_ansi_and_controls(malicious), "Mascot Art");
    }

    #[test]
    fn test_strip_two_byte_and_c1_escapes() {
        let input = "\x1bc\x1b=\x1b7\x1b8\u{009B}2J\u{009D}0;Title\x07Protected Art";
        assert_eq!(strip_ansi_and_controls(input), "Protected Art");
    }

    #[test]
    fn test_strip_c0_controls_preserving_newlines_and_tabs() {
        let input = "Line 1\x07\x08\r\n\tTabbed\x1f\x7f Line 2";
        assert_eq!(strip_ansi_and_controls(input), "Line 1\n\tTabbed Line 2");
    }

    #[test]
    fn test_unicode_and_mascot_symbols_preserved() {
        let art = "  ▲ █ ▓ ▒ ░ ▀ ▄ ❀ 🐮 Forgum ✨  ";
        assert_eq!(strip_ansi_and_controls(art), art);
    }
}
