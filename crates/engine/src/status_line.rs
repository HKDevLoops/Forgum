use crate::color;
use crate::cow;
use crate::fortune;

#[allow(dead_code)]
pub(crate) fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == 'm' {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[allow(dead_code)]
pub(crate) fn visible_length(s: &str) -> usize {
    strip_ansi(s).len()
}

pub fn render_status_line(max_len: usize) -> String {
    let data = match forgum_platform::data_dir() {
        Ok(d) => d,
        Err(_) => return String::new(),
    };

    let fortune_text = fortune::random_fortune(&data).unwrap_or_default();

    let cow_text = cow::load_cow("default", &data, "oo", "U", "\\\\");

    let composed = cow::compose_scene(&cow_text, &fortune_text);

    let flat: String = composed
        .chars()
        .map(|c| if c == '\n' { ' ' } else { c })
        .collect();

    let truncated: String = flat.chars().take(max_len).collect();

    let mut result = String::with_capacity(truncated.len() * 20);
    for (i, ch) in truncated.chars().enumerate() {
        let (r, g, b) = color::lolcat_color(i as f32, 0.0, 0.0, 0.0);
        result.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}"));
    }
    result.push_str("\x1b[0m");

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_sequences() {
        let input = "\x1b[38;2;255;0;0mhello\x1b[0m";
        assert_eq!(strip_ansi(input), "hello");
    }

    #[test]
    fn visible_length_counts_only_text() {
        let input = "\x1b[38;2;255;0;0mhi\x1b[0m";
        assert_eq!(visible_length(input), 2);
    }
}
