//! Kitty Graphics Protocol z-index plane rendering and Multiplexer status widgets.
//!
//! Provides:
//! 1. Hardware z-index composition (`z = -1` background, `z = 1` overlay) in modern GPU-accelerated
//!    terminals (Kitty, WezTerm, Ghostty), eliminating cursor conflicts during background animations.
//! 2. Native Tmux and Zellij status-bar widget generators.

use std::fmt::Write as _;

/// Check whether the current terminal environment advertises Kitty Graphics Protocol support.
pub fn is_kitty_graphics_supported() -> bool {
    let term = std::env::var("TERM").unwrap_or_default();
    let term_prog = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let kitty_win = std::env::var("KITTY_WINDOW_ID").is_ok();

    kitty_win
        || term.contains("kitty")
        || term_prog.eq_ignore_ascii_case("wezterm")
        || term_prog.eq_ignore_ascii_case("ghostty")
}

fn base64_encode(input: &[u8]) -> String {
    const TABLE: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Encode RGBA pixel buffer into Kitty Graphics Protocol commands placed on a specific z-index plane.
///
/// `z_index = -1`: Rendered behind the text layer (ideal for scenery and background pasture daemons).
/// `z_index = 0`: Inline with text.
/// `z_index = 1`: Overlay on top of text.
pub fn encode_kitty_z_plane(rgba_bytes: &[u8], width: u32, height: u32, z_index: i32) -> String {
    let b64 = base64_encode(rgba_bytes);

    let mut out = String::with_capacity(b64.len() + 64);
    let chunk_size = 4096;
    let chunks: Vec<&str> = b64
        .as_bytes()
        .chunks(chunk_size)
        .map(|c| std::str::from_utf8(c).unwrap_or_default())
        .collect();

    for (i, chunk) in chunks.iter().enumerate() {
        let is_last = i == chunks.len() - 1;
        let m = if is_last { 0 } else { 1 };

        if i == 0 {
            // First chunk transmits control parameters: format=32 (RGBA), size, z-index
            let _ = write!(
                out,
                "\x1b_Ga=T,f=32,s={width},v={height},z={z_index},m={m};{chunk}\x1b\\"
            );
        } else {
            // Continuation chunk
            let _ = write!(out, "\x1b_Gm={m};{chunk}\x1b\\");
        }
    }

    out
}

/// Escape sequence to clear all active Kitty graphics planes.
pub fn clear_kitty_graphics() -> &'static str {
    "\x1b_Ga=d,d=A\x1b\\"
}

/// Render status line formatted specifically for Tmux status-right.
pub fn render_tmux_status(mascot: Option<&str>, effect: Option<&str>, fps: Option<u16>) -> String {
    let mascot = mascot.unwrap_or("default");
    let effect = effect.unwrap_or("breathe");
    let fps = fps.unwrap_or(30);
    format!("#[fg=cyan]🐮 {mascot} #[fg=yellow]| #[fg=green]fx:{effect} #[fg=yellow]| #[fg=magenta]{fps}fps#[default]")
}

/// Render status line formatted specifically for Zellij status-bar widget.
pub fn render_zellij_status(mascot: Option<&str>, effect: Option<&str>, fps: Option<u16>) -> String {
    let mascot = mascot.unwrap_or("default");
    let effect = effect.unwrap_or("breathe");
    let fps = fps.unwrap_or(30);
    format!("\x1b[36m🐮 {mascot} \x1b[33m| \x1b[32m{effect} \x1b[33m| \x1b[35m{fps}Hz\x1b[0m")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_kitty_z_plane_format() {
        let test_rgba = [255u8, 0, 0, 255, 0, 255, 0, 255];
        let seq = encode_kitty_z_plane(&test_rgba, 2, 1, -1);
        assert!(seq.starts_with("\x1b_Ga=T,f=32,s=2,v=1,z=-1"));
        assert!(seq.ends_with("\x1b\\"));
    }

    #[test]
    fn test_tmux_and_zellij_format_non_empty() {
        let tmux = render_tmux_status(Some("tux"), Some("walk"), Some(240));
        assert!(tmux.contains("tux"));
        assert!(tmux.contains("walk"));

        let zellij = render_zellij_status(Some("dragon"), Some("fire"), Some(240));
        assert!(zellij.contains("dragon"));
        assert!(zellij.contains("fire"));
    }
}
