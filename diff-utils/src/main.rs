mod app;
mod highlight;
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
    /// UI color scheme: `dark` (default) or `light`. Press `t` in the app to toggle.
    #[arg(long, value_name = "SCHEME", default_value = "dark")]
    theme: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let theme = ColorScheme::parse(&cli.theme)
        .ok_or_else(|| anyhow::anyhow!("invalid --theme {:?} (expected dark or light)", cli.theme))?;
    app::run(cli.left.as_deref(), cli.right.as_deref(), theme)
}
