use crate::highlight::HighlightEngine;
use crate::theme::{ColorScheme, UiTheme};
use crate::ui;
use anyhow::{Context, Result};
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use diff_tool_core::{
    diff_lines, sibling_files, Entry, FileBrowser, NavigateTarget, SideBySide,
};
use ratatui::backend::CrosstermBackend;
use ratatui::text::Span;
use ratatui::Terminal;
use std::io::stdout;
use std::path::{Path, PathBuf};

/// Which of the two panels is currently focused.
pub const LEFT: usize = 0;
pub const RIGHT: usize = 1;

/// One half of the diff view.
pub struct Panel {
    pub path: Option<PathBuf>,
    /// Loaded text content of the current file, or an error message if it
    /// could not be read.
    pub content: Option<Result<String, String>>,
    /// Active when the panel has no file (or the user pressed `q` to close it).
    pub browser: Option<FileBrowser>,
    /// Per-line syntax-highlighted spans for the current file, indexed in
    /// source order (line 1 → index 0). `None` when the file has no known
    /// syntax or no file is loaded.
    pub highlighted: Option<Vec<Vec<Span<'static>>>>,
}

impl Panel {
    fn new() -> Self {
        Panel {
            path: None,
            content: None,
            browser: None,
            highlighted: None,
        }
    }

    pub fn open_browser(&mut self, root: Option<&Path>) {
        self.browser = FileBrowser::open(root).ok();
    }

    /// Load `path` as this panel's file and leave browser mode. The caller is
    /// responsible for refreshing syntax highlighting via `App::populate_highlight`.
    pub fn load(&mut self, path: PathBuf) {
        let result = std::fs::read_to_string(&path).map_err(|e| e.to_string());
        self.content = Some(result);
        self.path = Some(path);
        self.browser = None;
        self.highlighted = None;
    }

    /// Drop the current file and re-enter browser mode rooted at the file's
    /// parent (so the user can pick a sibling file quickly).
    pub fn close_file(&mut self) {
        let parent = self.path.as_ref().and_then(|p| p.parent()).map(|p| p.to_path_buf());
        let parent_ref = parent.as_deref();
        self.path = None;
        self.content = None;
        self.highlighted = None;
        self.open_browser(parent_ref);
    }

    pub fn has_file(&self) -> bool {
        self.path.is_some()
    }

    pub fn text(&self) -> Option<&str> {
        match self.content.as_ref()? {
            Ok(t) => Some(t.as_str()),
            Err(_) => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self.content.as_ref()? {
            Ok(_) => None,
            Err(e) => Some(e.as_str()),
        }
    }
}

/// Dropdown opened by clicking a panel's file-path title: lists sibling files
/// in the same directory so the user can switch without entering browser mode.
#[derive(Debug, Clone)]
pub struct FileSwitcher {
    pub panel: usize,
    pub entries: Vec<Entry>,
    pub selected: usize,
}

/// Top-level TUI state.
pub struct App {
    pub panels: [Panel; 2],
    pub focused: usize,
    pub scroll: usize,
    pub diff: SideBySide,
    pub show_help: bool,
    pub should_quit: bool,
    pub message: Option<String>,
    pub theme: UiTheme,
    pub highlight: HighlightEngine,
    /// When set, the focused panel's browser shows a path input line for typing
    /// or pasting a file/directory path.
    pub path_input: Option<String>,
    /// When set, a sibling-file dropdown is open for `FileSwitcher::panel`.
    pub file_switcher: Option<FileSwitcher>,
}

impl App {
    pub fn new(left: Option<&str>, right: Option<&str>, scheme: ColorScheme) -> Result<Self> {
        let theme = UiTheme::new(scheme);
        let mut panels = [Panel::new(), Panel::new()];

        if let Some(p) = left {
            panels[LEFT].load(PathBuf::from(p));
        } else {
            panels[LEFT].open_browser(None);
        }
        if let Some(p) = right {
            panels[RIGHT].load(PathBuf::from(p));
        } else {
            panels[RIGHT].open_browser(None);
        }

        let mut app = App {
            panels,
            focused: LEFT,
            scroll: 0,
            diff: SideBySide::default(),
            show_help: false,
            should_quit: false,
            message: Some(format!("theme: {}", theme.scheme.label())),
            theme,
            highlight: HighlightEngine::new(&theme),
            path_input: None,
            file_switcher: None,
        };
        app.populate_highlight(LEFT);
        app.populate_highlight(RIGHT);
        app.recompute_diff();
        Ok(app)
    }

    /// (Re)compute cached syntax-highlighted spans for panel `idx` based on its
    /// current file. Falls back to `None` (plain rendering) when the file has
    /// no recognized syntax.
    pub fn populate_highlight(&mut self, idx: usize) {
        let highlighted = {
            let panel = &self.panels[idx];
            let Some(path) = panel.path.as_deref() else {
                self.panels[idx].highlighted = None;
                return;
            };
            let Some(text) = panel.text() else {
                self.panels[idx].highlighted = None;
                return;
            };
            match self.highlight.syntax_for_path(path) {
                Some(syntax) => Some(self.highlight.highlight_text(&syntax, text)),
                None => None,
            }
        };
        self.panels[idx].highlighted = highlighted;
    }

    /// Recompute the side-by-side diff whenever either file changes.
    pub fn recompute_diff(&mut self) {
        match (self.panels[LEFT].text(), self.panels[RIGHT].text()) {
            (Some(l), Some(r)) => self.diff = diff_lines(l, r),
            _ => self.diff = SideBySide::default(),
        }
        // Clamp scroll to the new diff length.
        let max = self.diff.len().saturating_sub(1);
        if self.scroll > max {
            self.scroll = max;
        }
    }

    pub fn focus(&mut self, idx: usize) {
        if idx < 2 {
            self.focused = idx;
        }
    }

    pub fn toggle_focus(&mut self) {
        self.focused = 1 - self.focused;
    }

    /// Exchange the left and right panels (files, browsers, highlights) and
    /// refresh the diff.
    pub fn swap_panels(&mut self) {
        self.panels.swap(LEFT, RIGHT);
        self.recompute_diff();
        self.set_message("panels swapped");
    }

    pub fn focused_panel(&self) -> &Panel {
        &self.panels[self.focused]
    }

    pub fn focused_panel_mut(&mut self) -> &mut Panel {
        &mut self.panels[self.focused]
    }

    /// Move the diff scroll by `delta` rows (only meaningful in diff view).
    pub fn scroll_diff(&mut self, delta: isize) {
        let len = self.diff.len();
        if len == 0 {
            self.scroll = 0;
            return;
        }
        let mut next = self.scroll as isize + delta;
        if next < 0 {
            next = 0;
        }
        let max = (len - 1) as isize;
        if next > max {
            next = max;
        }
        self.scroll = next as usize;
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    pub fn path_input_active(&self) -> bool {
        self.path_input.is_some()
    }

    pub fn start_path_input(&mut self) {
        self.path_input = Some(String::new());
    }

    pub fn cancel_path_input(&mut self) {
        self.path_input = None;
    }

    /// Apply a typed or pasted path in the focused panel's browser.
    pub fn submit_path_input(&mut self) {
        let Some(input) = self.path_input.take() else {
            return;
        };
        let focused = self.focused;
        let Some(browser) = self.panels[focused].browser.as_mut() else {
            return;
        };

        match browser.navigate_target(&input) {
            Ok(NavigateTarget::Directory(dir)) => {
                if let Err(e) = browser.navigate_to_dir(&dir) {
                    self.path_input = Some(input);
                    self.set_message(e.to_string());
                } else {
                    self.set_message(format!("cd {}", dir.display()));
                }
            }
            Ok(NavigateTarget::File(path)) => {
                self.panels[focused].load(path);
                self.populate_highlight(focused);
                self.recompute_diff();
                self.scroll = 0;
            }
            Err(e) => {
                self.path_input = Some(input);
                self.set_message(e.to_string());
            }
        }
    }

    /// Navigate to a pasted path, or fill the path input for editing.
    pub fn paste_path(&mut self, text: &str) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        if self.focused_panel().browser.is_none() {
            return;
        }
        if self.path_input_active() {
            self.path_input = Some(trimmed.to_string());
        } else {
            self.path_input = Some(trimmed.to_string());
            self.submit_path_input();
        }
    }

    /// Switch between dark and light UI/syntax themes and refresh highlights.
    pub fn toggle_theme(&mut self) {
        self.theme = UiTheme::new(self.theme.scheme.toggle());
        self.highlight.set_ui_theme(&self.theme);
        self.populate_highlight(LEFT);
        self.populate_highlight(RIGHT);
        self.set_message(format!("theme: {}", self.theme.scheme.label()));
    }

    pub fn file_switcher_active(&self) -> bool {
        self.file_switcher.is_some()
    }

    /// Open a sibling-file dropdown for `panel` (must currently have a file).
    pub fn open_file_switcher(&mut self, panel: usize) {
        if panel >= 2 {
            return;
        }
        let Some(path) = self.panels[panel].path.clone() else {
            return;
        };
        match sibling_files(&path) {
            Ok(entries) if entries.is_empty() => {
                self.set_message("no sibling files in this directory");
            }
            Ok(entries) => {
                let selected = entries
                    .iter()
                    .position(|e| e.path == path)
                    .unwrap_or(0);
                self.focus(panel);
                self.cancel_path_input();
                self.file_switcher = Some(FileSwitcher {
                    panel,
                    entries,
                    selected,
                });
                self.set_message("pick a file — Enter/click to open, Esc to cancel");
            }
            Err(e) => self.set_message(e.to_string()),
        }
    }

    pub fn close_file_switcher(&mut self) {
        self.file_switcher = None;
    }

    pub fn move_file_switcher(&mut self, delta: isize) {
        let Some(switcher) = self.file_switcher.as_mut() else {
            return;
        };
        if switcher.entries.is_empty() {
            return;
        }
        let n = switcher.entries.len() as isize;
        let mut next = switcher.selected as isize + delta;
        if next < 0 {
            next = 0;
        }
        if next >= n {
            next = n - 1;
        }
        switcher.selected = next as usize;
    }

    /// Load the currently selected sibling file into the switcher's panel.
    pub fn confirm_file_switcher(&mut self) {
        let Some(switcher) = self.file_switcher.take() else {
            return;
        };
        let Some(entry) = switcher.entries.get(switcher.selected) else {
            return;
        };
        let path = entry.path.clone();
        let panel = switcher.panel;
        self.panels[panel].load(path);
        self.populate_highlight(panel);
        self.recompute_diff();
        self.scroll = 0;
        self.focus(panel);
        self.set_message(format!(
            "opened {}",
            self.panels[panel]
                .path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        ));
    }
}

pub fn run(left: Option<&str>, right: Option<&str>, scheme: ColorScheme) -> Result<()> {
    // Full-screen TUI: always emit ANSI colors even when NO_COLOR is set in the
    // parent environment (common in CI and cloud shells).
    crossterm::style::force_color_output(true);
    terminal::enable_raw_mode().context("enable raw mode")?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).context("enter alt screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(left, right, scheme)?;

    let result = main_loop(&mut terminal, &mut app);

    // Restore the terminal no matter what.
    disable_raw_mode_and_restore().ok();
    result
}

fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if !event::poll(std::time::Duration::from_millis(250))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) => {
                handle_key(app, key);
                if app.should_quit {
                    return Ok(());
                }
            }
            Event::Paste(text) => handle_paste(app, &text),
            Event::Mouse(mouse) => handle_mouse(app, mouse),
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    // File-switcher dropdown takes priority over other modes (except force-quit).
    if app.file_switcher_active() {
        match key.code {
            Esc | Char('q') => {
                app.close_file_switcher();
                app.set_message("file switch cancelled");
            }
            Char('Q') => {
                app.should_quit = true;
            }
            Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                app.should_quit = true;
            }
            Enter | Char('l') | Right => app.confirm_file_switcher(),
            Down | Char('j') => app.move_file_switcher(1),
            Up | Char('k') => app.move_file_switcher(-1),
            PageDown | Char('J') => app.move_file_switcher(10),
            PageUp | Char('K') => app.move_file_switcher(-10),
            Home | Char('g') => {
                if let Some(s) = app.file_switcher.as_mut() {
                    s.selected = 0;
                }
            }
            End | Char('G') => {
                if let Some(s) = app.file_switcher.as_mut() {
                    s.selected = s.entries.len().saturating_sub(1);
                }
            }
            Tab => {
                app.close_file_switcher();
                app.toggle_focus();
            }
            Char('?') => {
                app.show_help = !app.show_help;
            }
            _ => {}
        }
        return;
    }

    // Global keys.
    match key.code {
        Char('?') => {
            app.show_help = !app.show_help;
            return;
        }
        Tab => {
            app.cancel_path_input();
            app.toggle_focus();
            return;
        }
        Char('t') => {
            app.toggle_theme();
            return;
        }
        Char('s') => {
            app.swap_panels();
            return;
        }
        _ => {}
    }

    // `q` semantics: close the focused panel's file; if it has no file (already
    // browsing), quit the whole app.
    if matches!(key.code, Char('q')) {
        if app.path_input_active() {
            app.cancel_path_input();
            return;
        }
        if app.focused_panel().has_file() {
            app.focused_panel_mut().close_file();
            app.recompute_diff();
            app.scroll = 0;
            app.set_message(format!("panel {} file closed — pick a new file", side_name(app.focused)));
        } else {
            app.should_quit = true;
        }
        return;
    }

    // Force-quit the whole app: capital `Q` or Ctrl-C.
    if matches!(key.code, Char('Q')) {
        app.should_quit = true;
        return;
    }
    if matches!(key.code, Char('c'))
        && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL)
    {
        app.should_quit = true;
        return;
    }

    let focused = app.focused;
    let in_browser = app.panels[focused].browser.is_some();

    if app.path_input_active() {
        if in_browser {
            handle_path_input_key(app, key);
        } else {
            app.cancel_path_input();
        }
        return;
    }

    if in_browser {
        handle_browser_key(app, key.code);
    } else {
        handle_diff_key(app, key.code);
    }
}

fn handle_paste(app: &mut App, text: &str) {
    if app.focused_panel().browser.is_some() {
        app.paste_path(text);
    }
}

fn handle_path_input_key(app: &mut App, key: KeyEvent) {
    use KeyCode::*;

    match key.code {
        Esc => app.cancel_path_input(),
        Enter => app.submit_path_input(),
        Backspace => {
            if let Some(input) = app.path_input.as_mut() {
                input.pop();
            }
        }
        Char(c)
            if key.modifiers.is_empty() || key.modifiers == crossterm::event::KeyModifiers::SHIFT =>
        {
            app.path_input.get_or_insert_with(String::new).push(c);
        }
        _ => {}
    }
}

fn handle_diff_key(app: &mut App, code: KeyCode) {
    use KeyCode::*;
    match code {
        Down | Char('j') => app.scroll_diff(1),
        Up | Char('k') => app.scroll_diff(-1),
        PageDown | Char('J') => app.scroll_diff(10),
        PageUp | Char('K') => app.scroll_diff(-10),
        Home | Char('g') => app.scroll = 0,
        End | Char('G') => {
            let max = app.diff.len().saturating_sub(1);
            app.scroll = max;
        }
        Char('o') => {
            let focused = app.focused;
            app.open_file_switcher(focused);
        }
        _ => {}
    }
}

fn handle_browser_key(app: &mut App, code: KeyCode) {
    use KeyCode::*;
    let focused = app.focused;
    let Some(browser) = app.panels[focused].browser.as_mut() else {
        return;
    };

    match code {
        Down | Char('j') => browser.move_cursor(1),
        Up | Char('k') => browser.move_cursor(-1),
        PageDown | Char('J') => browser.move_cursor(10),
        PageUp | Char('K') => browser.move_cursor(-10),
        Home | Char('g') => browser.selected = 0,
        End | Char('G') => {
            browser.selected = browser.entries.len().saturating_sub(1);
        }
        Char('h') | Left | Backspace => {
            if let Err(e) = browser.go_up() {
                app.set_message(e.to_string());
            }
        }
        Char('H') => {
            if let Err(e) = browser.toggle_hidden() {
                app.set_message(e.to_string());
            }
        }
        Char('/') => app.start_path_input(),
        Char('l') | Right | Enter => {
            // Try to enter a directory first; otherwise load the selected file.
            match browser.enter_selected() {
                Ok(true) => {}
                Ok(false) => {
                    if let Some(path) = browser.selected_path() {
                        let path = path.to_path_buf();
                        app.panels[focused].load(path);
                        app.populate_highlight(focused);
                        app.recompute_diff();
                        app.scroll = 0;
                    }
                }
                Err(e) => app.set_message(e.to_string()),
            }
        }
        _ => {}
    }
}

fn handle_mouse(app: &mut App, mouse: crossterm::event::MouseEvent) {
    use crossterm::event::MouseEventKind;

    let (col, row) = (mouse.column, mouse.row);
    let (width, height) = match terminal_size() {
        Some(size) => size,
        None => return,
    };

    match mouse.kind {
        MouseEventKind::ScrollDown => {
            if app.file_switcher_active() {
                app.move_file_switcher(3);
            } else {
                app.scroll_diff(3);
            }
        }
        MouseEventKind::ScrollUp => {
            if app.file_switcher_active() {
                app.move_file_switcher(-3);
            } else {
                app.scroll_diff(-3);
            }
        }
        MouseEventKind::Down(_) => {
            // Clicks on an open dropdown select / confirm an entry.
            if let Some(hit) = ui::hit_test_file_switcher(app, col, row, width, height) {
                if let Some(switcher) = app.file_switcher.as_mut() {
                    if hit < switcher.entries.len() {
                        switcher.selected = hit;
                        app.confirm_file_switcher();
                    }
                }
                return;
            }

            // Click outside an open dropdown dismisses it.
            if app.file_switcher_active() {
                app.close_file_switcher();
                app.set_message("file switch cancelled");
                return;
            }

            let panel_idx = ui::panel_at_column(col, width);
            app.focus(panel_idx);

            // Click on a panel's file-path title opens the sibling-file dropdown.
            if ui::hit_test_path_title(app, panel_idx, col, row, width, height) {
                app.open_file_switcher(panel_idx);
                return;
            }

            // Click inside a browser navigates to the entry under the cursor.
            // Header occupies 2 rows (title + border), so content starts at row 2.
            let content_row = ui::panel_content_row(row, height);
            if let Some(content_row) = content_row {
                let list_offset = if app.path_input_active() && app.focused == panel_idx {
                    2
                } else {
                    1
                };
                if let Some(browser) = app.panels[panel_idx].browser.as_mut() {
                    // Browser draws a cwd line (and optional path input) above the list.
                    if content_row >= list_offset {
                        let idx = (content_row - list_offset) as usize;
                        if idx < browser.entries.len() {
                            browser.selected = idx;
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn terminal_size() -> Option<(u16, u16)> {
    crossterm::terminal::size().ok()
}

pub fn side_name(idx: usize) -> &'static str {
    if idx == LEFT { "left" } else { "right" }
}

fn disable_raw_mode_and_restore() -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
