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

    /// Resolve a user-typed or pasted path against the browser's current directory.
    /// Tilde (`~/…`) is expanded; relative paths are joined with `cwd`.
    pub fn resolve_path(&self, input: &str) -> Result<PathBuf, ResolveError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(ResolveError::Empty);
        }
        let expanded = expand_tilde(trimmed);
        let path = if expanded.is_absolute() {
            expanded
        } else {
            self.cwd.join(expanded)
        };
        if !path.exists() {
            return Err(ResolveError::NotFound(path));
        }
        Ok(path)
    }

    /// Navigate to an existing directory, keeping the browser open.
    pub fn navigate_to_dir(&mut self, dir: &Path) -> Result<(), BrowserError> {
        self.cwd = dir.to_path_buf();
        self.selected = 0;
        self.refresh()
    }

    /// Resolve `input` and classify it as a directory or file target.
    pub fn navigate_target(&self, input: &str) -> Result<NavigateTarget, ResolveError> {
        let path = self.resolve_path(input)?;
        if path.is_dir() {
            Ok(NavigateTarget::Directory(path))
        } else {
            Ok(NavigateTarget::File(path))
        }
    }
}

/// List directory entries for the path-title file switcher.
///
/// Includes a synthetic `..` parent entry (when a parent exists), then
/// directories and files (hidden names omitted), sorted dirs-first.
pub fn switcher_entries(dir: &Path) -> Result<Vec<Entry>, BrowserError> {
    let mut entries = Vec::new();
    if let Some(parent) = dir.parent().filter(|p| !p.as_os_str().is_empty()) {
        entries.push(Entry {
            path: parent.to_path_buf(),
            name: "..".to_string(),
            is_dir: true,
        });
    }
    entries.extend(list_dir(dir, false)?);
    Ok(entries)
}

/// Directory that contains `path` (or `.` when `path` has no parent).
pub fn parent_dir(path: &Path) -> PathBuf {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Walk from `path` upward and return the first existing directory.
///
/// If `path` itself is an existing directory, it is returned. Intended for CLI
/// fallback when a requested file or directory path does not exist.
pub fn existing_ancestor_dir(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);
    while let Some(p) = current {
        if p.is_dir() {
            return Some(p.to_path_buf());
        }
        current = p
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty());
    }
    None
}

/// List files (not directories) in the same directory as `path`.
///
/// Hidden files are omitted. Prefer [`switcher_entries`] for the TUI dropdown,
/// which also includes directories and a parent entry.
pub fn sibling_files(path: &Path) -> Result<Vec<Entry>, BrowserError> {
    let parent = parent_dir(path);
    let entries = list_dir(&parent, false)?;
    Ok(entries.into_iter().filter(|e| !e.is_dir).collect())
}

/// Outcome of resolving a path string for navigation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigateTarget {
    Directory(PathBuf),
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolveError {
    Empty,
    NotFound(PathBuf),
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::Empty => write!(f, "empty path"),
            ResolveError::NotFound(p) => write!(f, "path not found: {}", p.display()),
        }
    }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    } else if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home);
        }
    }
    PathBuf::from(path)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "diff-tool-browser-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolve_relative_file_in_cwd() {
        let root = temp_dir();
        let file = root.join("hello.txt");
        fs::write(&file, "hi").unwrap();
        let browser = FileBrowser::open(Some(&root)).unwrap();
        let target = browser.navigate_target("hello.txt").unwrap();
        assert_eq!(target, NavigateTarget::File(file));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_absolute_directory() {
        let root = temp_dir();
        let sub = root.join("subdir");
        fs::create_dir(&sub).unwrap();
        let browser = FileBrowser::open(Some(&root)).unwrap();
        let target = browser.navigate_target(sub.to_str().unwrap()).unwrap();
        assert_eq!(target, NavigateTarget::Directory(sub));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn resolve_missing_path_errors() {
        let root = temp_dir();
        let browser = FileBrowser::open(Some(&root)).unwrap();
        let err = browser.navigate_target("no-such-file.txt").unwrap_err();
        assert!(matches!(err, ResolveError::NotFound(_)));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn navigate_to_dir_updates_cwd() {
        let root = temp_dir();
        let sub = root.join("nested");
        fs::create_dir(&sub).unwrap();
        let mut browser = FileBrowser::open(Some(&root)).unwrap();
        browser.navigate_to_dir(&sub).unwrap();
        assert_eq!(browser.cwd, sub);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sibling_files_lists_files_not_dirs() {
        let root = temp_dir();
        fs::write(root.join("a.txt"), "a").unwrap();
        fs::write(root.join("b.txt"), "b").unwrap();
        fs::create_dir(root.join("subdir")).unwrap();
        fs::write(root.join(".hidden"), "h").unwrap();
        let siblings = sibling_files(&root.join("a.txt")).unwrap();
        let names: Vec<_> = siblings.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["a.txt", "b.txt"]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn switcher_entries_include_parent_dirs_and_files() {
        let root = temp_dir();
        let nested = root.join("nested");
        fs::create_dir(&nested).unwrap();
        fs::write(nested.join("a.txt"), "a").unwrap();
        fs::create_dir(nested.join("sub")).unwrap();
        let entries = switcher_entries(&nested).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names.first().copied(), Some(".."));
        assert!(names.contains(&"a.txt"));
        assert!(names.contains(&"sub"));
        assert_eq!(entries[0].path, root);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn existing_ancestor_dir_returns_self_for_directory() {
        let root = temp_dir();
        assert_eq!(existing_ancestor_dir(&root).as_deref(), Some(root.as_path()));
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn existing_ancestor_dir_walks_past_missing_components() {
        let root = temp_dir();
        let missing = root.join("nope").join("also").join("file.txt");
        assert_eq!(
            existing_ancestor_dir(&missing).as_deref(),
            Some(root.as_path())
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn existing_ancestor_dir_skips_existing_file() {
        let root = temp_dir();
        let file = root.join("note.txt");
        fs::write(&file, "hi").unwrap();
        assert_eq!(existing_ancestor_dir(&file).as_deref(), Some(root.as_path()));
        let _ = fs::remove_dir_all(&root);
    }
}
