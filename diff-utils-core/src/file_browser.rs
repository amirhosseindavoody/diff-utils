//! A minimal file browser model used by the TUI when a panel has no file
//! selected (or after the user closes one with `q`).

use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("cannot read directory {0}: {1}")]
    Read(PathBuf, #[source] std::io::Error),
}

/// One entry in a directory listing.
#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

/// A browsing cursor over a single directory.
#[derive(Debug, Clone)]
pub struct FileBrowser {
    pub cwd: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: usize,
    /// Optional filter so only files are pickable; directories are still
    /// navigable.
    pub show_hidden: bool,
}

impl FileBrowser {
    /// Open a browser rooted at `dir` (defaults to the current directory if
    /// `dir` is `None` or doesn't exist).
    pub fn open(dir: Option<&Path>) -> Result<Self, BrowserError> {
        let cwd = match dir {
            Some(p) if p.exists() => p.to_path_buf(),
            _ => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        };
        let mut browser = FileBrowser {
            cwd,
            entries: Vec::new(),
            selected: 0,
            show_hidden: false,
        };
        browser.refresh()?;
        Ok(browser)
    }

    /// Re-read the current directory.
    pub fn refresh(&mut self) -> Result<(), BrowserError> {
        self.entries = list_dir(&self.cwd, self.show_hidden)?;
        // Keep selection in range, preferring to land on the first file.
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
        Ok(())
    }

    /// Navigate into the parent directory.
    pub fn go_up(&mut self) -> Result<(), BrowserError> {
        if let Some(parent) = self.cwd.parent() {
            let previous = self.cwd.clone();
            self.cwd = parent.to_path_buf();
            self.refresh()?;
            // Restore focus onto the directory we came from, if present.
            if let Some(pos) = self
                .entries
                .iter()
                .position(|e| e.path == previous || e.name == previous.to_string_lossy())
            {
                self.selected = pos;
            }
        }
        Ok(())
    }

    /// Navigate into the currently selected entry if it is a directory.
    pub fn enter_selected(&mut self) -> Result<bool, BrowserError> {
        let Some(entry) = self.entries.get(self.selected) else {
            return Ok(false);
        };
        if !entry.is_dir {
            return Ok(false);
        }
        self.cwd = entry.path.clone();
        self.selected = 0;
        self.refresh()?;
        Ok(true)
    }

    /// The path of the currently selected entry, if any.
    pub fn selected_path(&self) -> Option<&Path> {
        self.entries.get(self.selected).map(|e| e.path.as_path())
    }

    pub fn move_cursor(&mut self, delta: isize) {
        if self.entries.is_empty() {
            return;
        }
        let n = self.entries.len() as isize;
        let mut next = self.selected as isize + delta;
        if next < 0 {
            next = 0;
        }
        if next >= n {
            next = n - 1;
        }
        self.selected = next as usize;
    }

    pub fn toggle_hidden(&mut self) -> Result<(), BrowserError> {
        self.show_hidden = !self.show_hidden;
        self.refresh()
    }
}

/// List directory entries, sorted: directories first, then files, alphabetical.
fn list_dir(dir: &Path, show_hidden: bool) -> Result<Vec<Entry>, BrowserError> {
    let mut dirs: Vec<Entry> = Vec::new();
    let mut files: Vec<Entry> = Vec::new();

    let read = fs::read_dir(dir).map_err(|e| BrowserError::Read(dir.to_path_buf(), e))?;
    for entry in read {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') {
            continue;
        }
        let is_dir = entry
            .file_type()
            .map(|t| t.is_dir())
            .unwrap_or(false);
        let path = entry.path();
        let item = Entry { path, name, is_dir };
        if is_dir {
            dirs.push(item);
        } else {
            files.push(item);
        }
    }

    dirs.sort_by_key(|a| a.name.to_lowercase());
    files.sort_by_key(|a| a.name.to_lowercase());
    dirs.extend(files);
    Ok(dirs)
}
