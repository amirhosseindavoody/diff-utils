//! Shared library for diff-utils: side-by-side diff computation and a file
//! browser model used by the TUI.

pub mod diff;
pub mod file_browser;

pub use diff::{diff_lines, DiffRow, DiffSide, DiffStats, RowKind, SideBySide};
pub use file_browser::{Entry, FileBrowser};
