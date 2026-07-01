//! Syntax highlighting for the diff panels, built on `syntect`.
//!
//! Uses the pure-Rust `regex-fancy` engine (no oniguruma C dependency) so the
//! build stays friendly to conda/pixi packaging. A small custom `.log` syntax
//! is registered on top of syntect's default syntax set so log files get
//! colored timestamps and log levels.

use crate::theme::UiTheme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use std::path::Path;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxDefinition, SyntaxReference, SyntaxSet};

/// A minimal Sublime-style syntax for generic log files: timestamps and the
/// common log levels (ERROR/WARN/INFO/DEBUG) are colored via standard scopes.
const LOG_SYNTAX: &str = r#"%YAML 1.2
---
name: Log
file_extensions: [log, syslog, out]
scope: text.log
contexts:
  main:
    - match: '\b\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:?\d{2})?\b'
      scope: comment.other.timestamp.log
    - match: '\b\d{2}:\d{2}:\d{2}(?:\.\d+)?\b'
      scope: comment.other.timestamp.log
    - match: '\b(?i:ERROR|ERR|FATAL|CRITICAL|SEVERE|PANIC)\b'
      scope: keyword.control.error.log
    - match: '\b(?i:WARN(?:ING)?)\b'
      scope: keyword.control.warn.log
    - match: '\b(?i:INFO|NOTICE)\b'
      scope: keyword.control.info.log
    - match: '\b(?i:DEBUG|TRACE|FINE(?:ST|R)?)\b'
      scope: keyword.control.debug.log
    - match: '\b([A-Z][A-Z0-9_]{2,})\b'
      scope: support.constant.log
"#;

/// Owns the syntect state needed to highlight any supported file.
pub struct HighlightEngine {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl HighlightEngine {
    pub fn new(ui_theme: &UiTheme) -> Self {
        let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
        if let Ok(log_def) = SyntaxDefinition::load_from_str(LOG_SYNTAX, false, Some("log")) {
            builder.add(log_def);
        }
        let syntax_set = builder.build();
        let theme = load_syntect_theme(ui_theme);

        HighlightEngine { syntax_set, theme }
    }

    pub fn set_ui_theme(&mut self, ui_theme: &UiTheme) {
        self.theme = load_syntect_theme(ui_theme);
    }

    /// Pick a syntax reference for `path`, by extension then first-line hint.
    /// Returns `None` for unrecognized files (caller falls back to plain text).
    pub fn syntax_for_path(&self, path: &Path) -> Option<SyntaxReference> {
        match self.syntax_set.find_syntax_for_file(path) {
            Ok(Some(syntax)) => Some(syntax.clone()),
            _ => self
                .syntax_set
                .find_syntax_by_extension(path.extension()?.to_str()?)
                .cloned(),
        }
    }

    /// Highlight every line of `text`, returning per-line styled `Span`s
    /// indexed in source order (line 1 → index 0). Multi-line constructs
    /// (e.g. Python triple-quoted strings) stay correct because the
    /// `HighlightLines` state is advanced sequentially over the whole file.
    pub fn highlight_text(&self, syntax: &SyntaxReference, text: &str) -> Vec<Vec<Span<'static>>> {
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut out = Vec::with_capacity(text.lines().count());
        for line in text.lines() {
            let regions = highlighter.highlight_line(line, &self.syntax_set);
            let spans = match regions {
                Ok(regions) => regions
                    .into_iter()
                    .map(|(style, text)| Span::styled(text.to_string(), to_tui_style(style)))
                    .collect(),
                Err(_) => vec![Span::raw(line.to_string())],
            };
            out.push(spans);
        }
        out
    }
}

/// Convert a syntect `Style` to a ratatui `Style`. Only foreground color and
/// font modifiers are carried over; background is left unset so the diff row's
/// background highlight (applied in `ui.rs`) shows through.
fn to_tui_style(style: syntect::highlighting::Style) -> Style {
    let mut modifier = Modifier::empty();
    if style.font_style.contains(FontStyle::BOLD) {
        modifier |= Modifier::BOLD;
    }
    if style.font_style.contains(FontStyle::ITALIC) {
        modifier |= Modifier::ITALIC;
    }
    if style.font_style.contains(FontStyle::UNDERLINE) {
        modifier |= Modifier::UNDERLINED;
    }
    Style::default()
        .fg(color_to_tui(style.foreground))
        .add_modifier(modifier)
}

fn color_to_tui(c: syntect::highlighting::Color) -> Color {
    Color::Rgb(c.r, c.g, c.b)
}

fn load_syntect_theme(ui_theme: &UiTheme) -> Theme {
    let theme_set = ThemeSet::load_defaults();
    let fallbacks: &[&str] = match ui_theme.scheme {
        crate::theme::ColorScheme::Dark => &[
            "base16-ocean.dark",
            "base16-eighties.dark",
            "Inspired GitHub",
        ],
        crate::theme::ColorScheme::Light => &[
            "Solarized (light)",
            "GitHub",
            "Inspired GitHub",
        ],
    };
    fallbacks
        .iter()
        .find_map(|name| theme_set.themes.get(*name))
        .or_else(|| theme_set.themes.get(ui_theme.syntect_theme))
        .or_else(|| theme_set.themes.values().next())
        .cloned()
        .expect("syntect default theme set is non-empty")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{ColorScheme, UiTheme};

    #[test]
    fn syntect_themes_load_for_dark_and_light() {
        for scheme in [ColorScheme::Dark, ColorScheme::Light] {
            let ui_theme = UiTheme::new(scheme);
            let _engine = HighlightEngine::new(&ui_theme);
        }
    }
}
