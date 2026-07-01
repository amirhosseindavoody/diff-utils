//! Terminal capability probes used before the full-screen TUI starts.
//!
//! The most important probe is OSC 11 (background color query). VS Code,
//! iTerm2, Ghostty, and most modern terminals respond with the integrated
//! theme's background RGB so we can pick a matching UI palette without
//! requiring `--theme light` on every launch.

use crate::theme::ColorScheme;
use std::io::{self, IsTerminal, Read, Write};
use std::time::{Duration, Instant};

/// Pick a color scheme that matches the terminal background when possible.
///
/// Falls back to [`ColorScheme::Dark`] when stdout is not a TTY or the probe
/// times out (CI, pipes, very old terminals).
pub fn detect_color_scheme() -> ColorScheme {
    if let Some(scheme) = scheme_from_colorfgbg() {
        return scheme;
    }

    if !io::stdout().is_terminal() {
        return ColorScheme::Dark;
    }

    let _ = crossterm::terminal::enable_raw_mode();
    let scheme = query_background_color(Duration::from_millis(100))
        .map(scheme_from_luminance)
        .unwrap_or(ColorScheme::Dark);
    let _ = crossterm::terminal::disable_raw_mode();
    scheme
}

fn scheme_from_colorfgbg() -> Option<ColorScheme> {
    let value = std::env::var("COLORFGBG").ok()?;
    let bg = value.split(';').nth(1)?.parse::<i32>().ok()?;
    Some(scheme_from_colorfgbg_index(bg))
}

fn scheme_from_colorfgbg_index(bg: i32) -> ColorScheme {
    // rxvt-style convention: background palette index >= 8 means a light terminal.
    if bg >= 8 {
        ColorScheme::Light
    } else {
        ColorScheme::Dark
    }
}

fn query_background_color(timeout: Duration) -> Option<(u8, u8, u8)> {
    let mut stdout = io::stdout();
    stdout.write_all(b"\x1b]11;?\x07").ok()?;
    stdout.flush().ok()?;

    let mut buf = Vec::with_capacity(64);
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        match read_stdin_byte(timeout.saturating_sub(Instant::now().elapsed())) {
            Some(b) => {
                buf.push(b);
                if b == 0x07 {
                    break;
                }
                if b == b'\\' && buf.len() >= 2 && buf[buf.len() - 2] == 0x1b {
                    break;
                }
            }
            None => break,
        }
    }

    parse_osc11_response(&buf)
}

#[cfg(unix)]
fn read_stdin_byte(timeout: Duration) -> Option<u8> {
    use std::os::unix::io::AsRawFd;

    let mut stdin = io::stdin();
    let fd = stdin.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 {
        return None;
    }
    unsafe {
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }

    let mut byte = [0u8; 1];
    let deadline = Instant::now() + timeout;
    let result = loop {
        if Instant::now() >= deadline {
            break None;
        }
        match stdin.read(&mut byte) {
            Ok(0) => break None,
            Ok(1) => break Some(byte[0]),
            Ok(_) => continue,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(_) => break None,
        }
    };

    unsafe {
        libc::fcntl(fd, libc::F_SETFL, flags);
    }
    result
}

#[cfg(not(unix))]
fn read_stdin_byte(_timeout: Duration) -> Option<u8> {
    None
}

/// Parse an OSC 11 response such as `ESC ] 11 ; rgb:ffff/ffff/ffff BEL`.
fn parse_osc11_response(buf: &[u8]) -> Option<(u8, u8, u8)> {
    let text = std::str::from_utf8(buf).ok()?;
    let payload = text
        .strip_prefix("\x1b]11;")
        .or_else(|| text.strip_prefix("\x1b]11:"))?;

    if let Some(hex) = payload.strip_prefix('#') {
        return parse_hex_color(hex.trim_end_matches(|c| c == '\x07' || c == '\\'));
    }

    let rgb_part = payload
        .strip_prefix("rgb:")
        .unwrap_or(payload)
        .trim_end_matches(|c| c == '\x07' || c == '\\');

    let parts: Vec<&str> = rgb_part.split('/').collect();
    if parts.len() == 3 {
        let r = parse_rgb_component(parts[0])?;
        let g = parse_rgb_component(parts[1])?;
        let b = parse_rgb_component(parts[2])?;
        return Some((r, g, b));
    }

    None
}

fn parse_rgb_component(component: &str) -> Option<u8> {
    let trimmed = component.trim();
    if trimmed.len() <= 2 {
        return u8::from_str_radix(trimmed, 16).ok();
    }
    // 16-bit components repeat the high byte (e.g. ffff -> ff).
    let high = &trimmed[..2];
    u8::from_str_radix(high, 16).ok()
}

fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim();
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some((r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some((r * 17, g * 17, b * 17))
        }
        _ => None,
    }
}

/// Rec. 601 luminance; values above ~128 indicate a light background.
fn scheme_from_luminance(rgb: (u8, u8, u8)) -> ColorScheme {
    let (r, g, b) = rgb;
    let lum = 0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b);
    if lum > 127.5 {
        ColorScheme::Light
    } else {
        ColorScheme::Dark
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_osc11_16bit_rgb() {
        let buf = b"\x1b]11;rgb:ffff/ffff/ffff\x07";
        assert_eq!(parse_osc11_response(buf), Some((255, 255, 255)));
    }

    #[test]
    fn parse_osc11_dark_background() {
        let buf = b"\x1b]11;rgb:1818/1818/1818\x07";
        assert_eq!(parse_osc11_response(buf), Some((0x18, 0x18, 0x18)));
        assert_eq!(scheme_from_luminance((0x18, 0x18, 0x18)), ColorScheme::Dark);
    }

    #[test]
    fn parse_osc11_hex_form() {
        let buf = b"\x1b]11;#ffffff\x07";
        assert_eq!(parse_osc11_response(buf), Some((255, 255, 255)));
    }

    #[test]
    fn luminance_classifies_light_and_dark() {
        assert_eq!(scheme_from_luminance((255, 255, 255)), ColorScheme::Light);
        assert_eq!(scheme_from_luminance((24, 24, 24)), ColorScheme::Dark);
    }

    #[test]
    fn colorfgbg_light_when_bg_index_high() {
        assert_eq!(scheme_from_colorfgbg_index(15), ColorScheme::Light);
        assert_eq!(scheme_from_colorfgbg_index(0), ColorScheme::Dark);
    }
}
