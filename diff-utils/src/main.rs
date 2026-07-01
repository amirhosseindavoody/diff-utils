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
/// Pass zero, one, or two file paths. Any panel without a file opens a file
/// browser so you can pick one interactively.
#[derive(Parser)]
#[command(name = "diff-utils", version, about = "Side-by-side file diff TUI")]
struct Cli {
    /// First file (left panel). If omitted, the left panel starts in browser mode.
    left: Option<String>,
    /// Second file (right panel). If omitted, the right panel starts in browser mode.
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
