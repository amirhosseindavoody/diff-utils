//! UI color scheme for the TUI (ratatui widgets and diff row backgrounds).
//!
//! Syntax highlighting colors come from syntect; this module covers everything
//! else: borders, status bar, file browser, help overlay, and diff backgrounds.

use ratatui::style::Color;

/// Light or dark UI palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

impl ColorScheme {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "dark" => Some(Self::Dark),
            "light" => Some(Self::Light),
            _ => None,
        }
    }

    pub fn toggle(self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }
}

/// Ratatui colors and syntect theme name for one color scheme.
#[derive(Debug, Clone, Copy)]
pub struct UiTheme {
    pub scheme: ColorScheme,
    pub syntect_theme: &'static str,
    pub divider: Color,
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub line_number: Color,
    pub hint: Color,
    pub status_separator: Color,
    pub status_focused: Color,
    pub stat_added: Color,
    pub stat_removed: Color,
    pub stat_changed: Color,
    pub stat_equal: Color,
    pub browsing: Color,
    pub message: Color,
    pub error: Color,
    pub browser_cwd_label: Color,
    pub browser_cwd_path: Color,
    pub browser_highlight_bg: Color,
    pub browser_dir: Color,
    pub browser_file: Color,
    pub diff_equal: Color,
    pub diff_blank: Color,
    pub diff_added_bg: Color,
    pub diff_removed_bg: Color,
    pub diff_changed_bg: Color,
    pub help_bg: Color,
}

impl UiTheme {
    pub fn new(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::Dark => Self::dark(),
            ColorScheme::Light => Self::light(),
        }
    }

    pub fn dark() -> Self {
        Self {
            scheme: ColorScheme::Dark,
            syntect_theme: "base16-ocean.dark",
            divider: Color::DarkGray,
            border_focused: Color::Cyan,
            border_unfocused: Color::DarkGray,
            line_number: Color::DarkGray,
            hint: Color::DarkGray,
            status_separator: Color::DarkGray,
            status_focused: Color::Cyan,
            stat_added: Color::Green,
            stat_removed: Color::Red,
            stat_changed: Color::Yellow,
            stat_equal: Color::Gray,
            browsing: Color::Yellow,
            message: Color::Magenta,
            error: Color::Red,
            browser_cwd_label: Color::DarkGray,
            browser_cwd_path: Color::Yellow,
            browser_highlight_bg: Color::DarkGray,
            browser_dir: Color::Blue,
            browser_file: Color::White,
            diff_equal: Color::Gray,
            diff_blank: Color::DarkGray,
            diff_added_bg: Color::Rgb(0, 48, 0),
            diff_removed_bg: Color::Rgb(48, 0, 0),
            diff_changed_bg: Color::Rgb(48, 36, 0),
            help_bg: Color::Black,
        }
    }

    pub fn light() -> Self {
        Self {
            scheme: ColorScheme::Light,
            syntect_theme: "GitHub",
            divider: Color::Gray,
            border_focused: Color::Blue,
            border_unfocused: Color::Gray,
            line_number: Color::Rgb(100, 100, 100),
            hint: Color::Rgb(80, 80, 80),
            status_separator: Color::Gray,
            status_focused: Color::Blue,
            stat_added: Color::Rgb(0, 120, 0),
            stat_removed: Color::Rgb(180, 0, 0),
            stat_changed: Color::Rgb(160, 90, 0),
            stat_equal: Color::Rgb(80, 80, 80),
            browsing: Color::Rgb(160, 90, 0),
            message: Color::Magenta,
            error: Color::Red,
            browser_cwd_label: Color::Rgb(100, 100, 100),
            browser_cwd_path: Color::Rgb(160, 90, 0),
            browser_highlight_bg: Color::Rgb(210, 210, 210),
            browser_dir: Color::Blue,
            browser_file: Color::Black,
            diff_equal: Color::Rgb(60, 60, 60),
            diff_blank: Color::Rgb(120, 120, 120),
            diff_added_bg: Color::Rgb(180, 245, 180),
            diff_removed_bg: Color::Rgb(255, 190, 190),
            diff_changed_bg: Color::Rgb(255, 240, 160),
            help_bg: Color::Rgb(245, 245, 245),
        }
    }

    pub fn diff_bg(&self, kind: diff_tool_core::RowKind) -> Option<Color> {
        use diff_tool_core::RowKind;
        match kind {
            RowKind::Added => Some(self.diff_added_bg),
            RowKind::Removed => Some(self.diff_removed_bg),
            RowKind::Changed => Some(self.diff_changed_bg),
            _ => None,
        }
    }
}
