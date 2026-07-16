mod app;
mod highlight;
mod terminal;
mod theme;
mod ui;

use anyhow::Result;
use clap::Parser;
use theme::ColorScheme;

/// Side-by-side file diff TUI.
///
/// Pass zero, one, or two paths (files or directories). A single file opens on
/// both panels; a single directory opens a file picker rooted there on both
/// panels. With two paths, each panel opens its own file or directory picker.
/// Missing paths open a picker at the nearest existing parent directory.
#[derive(Parser)]
#[command(name = "diff-tool", version, about = "Side-by-side file diff TUI")]
struct Cli {
    /// First path (left panel). A file is loaded; a directory opens a picker.
    /// With only this argument, both panels use the same path.
    left: Option<String>,
    /// Second path (right panel). A file is loaded; a directory opens a picker.
    /// If omitted when `left` is set, the left path is mirrored to both panels.
    right: Option<String>,
    /// UI color scheme: `dark` or `light`. When omitted, matches the terminal
    /// background (OSC 11 probe). Press `t` in the app to toggle.
    #[arg(long, value_name = "SCHEME")]
    theme: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let theme = match cli.theme.as_deref() {
        Some(name) => ColorScheme::parse(name).ok_or_else(|| {
            anyhow::anyhow!("invalid --theme {name:?} (expected dark or light)")
        })?,
        None => terminal::detect_color_scheme(),
    };
    app::run(cli.left.as_deref(), cli.right.as_deref(), theme)
}
