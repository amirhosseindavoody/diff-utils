use crate::app::{side_name, App, LEFT, RIGHT};
use crate::theme::UiTheme;
use diff_tool_core::{abbreviated_path_titles, Entry, RowKind, SideBySide};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

/// Rows reserved for each panel's path title / top border.
pub const PANEL_HEADER_ROWS: u16 = 2;

/// Top-level draw.
pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = app.theme;
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let body = chunks[0];
    let status_bar = chunks[1];

    let (left, divider, right) = split_panels(body);

    let (left_path, right_path) = abbreviated_path_titles(
        app.panels[LEFT].path.as_deref(),
        app.panels[RIGHT].path.as_deref(),
    );

    draw_panel(f, app, LEFT, left, left_path.as_deref(), theme);
    draw_divider(f, divider, theme);
    draw_panel(f, app, RIGHT, right, right_path.as_deref(), theme);
    draw_status(f, app, status_bar, theme);

    if app.file_switcher.is_some() {
        draw_file_switcher(f, app, body, theme);
    }

    if app.show_help {
        draw_help(f, area, theme);
    }
}

/// Which panel contains terminal column `col` for the current width.
pub fn panel_at_column(col: u16, width: u16) -> usize {
    let body = Rect {
        x: 0,
        y: 0,
        width,
        height: 1,
    };
    let (left, _divider, right) = split_panels(body);
    if col >= right.x {
        RIGHT
    } else if col < left.x + left.width {
        LEFT
    } else {
        // Divider: treat as left for focus purposes.
        LEFT
    }
}

/// True when `(col, row)` hits the path title of `panel` (panel must have a file).
pub fn hit_test_path_title(
    app: &App,
    panel: usize,
    col: u16,
    row: u16,
    width: u16,
    height: u16,
) -> bool {
    if !app.panels[panel].has_file() {
        return false;
    }
    let Some(layout) = layout_geometry(width, height) else {
        return false;
    };
    let panel_area = if panel == LEFT {
        layout.left
    } else {
        layout.right
    };
    row < PANEL_HEADER_ROWS
        && col >= panel_area.x
        && col < panel_area.x.saturating_add(panel_area.width)
}

/// If `(col, row)` hits an open file-switcher list entry, return its index.
pub fn hit_test_file_switcher(
    app: &App,
    col: u16,
    row: u16,
    width: u16,
    height: u16,
) -> Option<usize> {
    let switcher = app.file_switcher.as_ref()?;
    let layout = layout_geometry(width, height)?;
    let area = file_switcher_area(layout, switcher.panel, switcher.entries.len())?;
    if col < area.x || col >= area.x.saturating_add(area.width) {
        return None;
    }
    if row < area.y || row >= area.y.saturating_add(area.height) {
        return None;
    }
    // Account for the bordered block: title row + bottom border.
    let inner_y = area.y.saturating_add(1);
    let inner_h = area.height.saturating_sub(2);
    if row < inner_y || row >= inner_y.saturating_add(inner_h) {
        return None;
    }
    Some((row - inner_y) as usize)
}

/// Convert a terminal row to a 0-based row within the panel content area.
pub fn panel_content_row(row: u16, height: u16) -> Option<u16> {
    if height == 0 {
        return None;
    }
    let body_h = height.saturating_sub(1); // status bar
    if row >= body_h || row < PANEL_HEADER_ROWS {
        return None;
    }
    Some(row - PANEL_HEADER_ROWS)
}

struct LayoutGeometry {
    left: Rect,
    right: Rect,
}

fn layout_geometry(width: u16, height: u16) -> Option<LayoutGeometry> {
    if width == 0 || height == 0 {
        return None;
    }
    let body = Rect {
        x: 0,
        y: 0,
        width,
        height: height.saturating_sub(1),
    };
    let (left, _divider, right) = split_panels(body);
    Some(LayoutGeometry { left, right })
}

fn file_switcher_area(layout: LayoutGeometry, panel: usize, entry_count: usize) -> Option<Rect> {
    let panel_area = if panel == LEFT {
        layout.left
    } else {
        layout.right
    };
    if panel_area.width < 4 || panel_area.height <= PANEL_HEADER_ROWS {
        return None;
    }
    let max_list = panel_area
        .height
        .saturating_sub(PANEL_HEADER_ROWS)
        .saturating_sub(2) // top+bottom border of dropdown
        .max(1);
    let list_h = (entry_count as u16).clamp(1, max_list);
    let height = list_h.saturating_add(2); // borders
    Some(Rect {
        x: panel_area.x,
        y: PANEL_HEADER_ROWS,
        width: panel_area.width,
        height,
    })
}

fn split_panels(area: Rect) -> (Rect, Rect, Rect) {
    let total = area.width;
    let divider_w: u16 = 1;
    let left_w = total.saturating_sub(divider_w) / 2;
    let right_w = total.saturating_sub(divider_w) - left_w;
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(left_w),
            Constraint::Length(divider_w),
            Constraint::Length(right_w),
        ])
        .split(area);
    (chunks[0], chunks[1], chunks[2])
}

fn draw_divider(f: &mut Frame, area: Rect, theme: UiTheme) {
    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(theme.divider));
    f.render_widget(block, area);
}

fn draw_panel(
    f: &mut Frame,
    app: &mut App,
    idx: usize,
    area: Rect,
    path_display: Option<&str>,
    theme: UiTheme,
) {
    let focused = app.focused == idx;
    let panel = &app.panels[idx];

    let title = panel_title(idx, focused, path_display);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);
    let header_area = inner[0];
    let content_area = inner[1];

    let border_style = if focused {
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(theme.border_focused)
    } else {
        Style::default().fg(theme.border_unfocused)
    };

    let header = Block::default()
        .borders(Borders::TOP)
        .title(title)
        .border_style(border_style);
    f.render_widget(header, header_area);

    if panel.browser.is_some() {
        draw_browser(f, app, idx, content_area, focused, theme);
    } else {
        draw_file_content(f, app, idx, content_area, theme);
    }
}

fn panel_title(idx: usize, focused: bool, path_display: Option<&str>) -> String {
    let marker = if focused { "◀" } else { " " };
    let side = side_name(idx);
    match path_display {
        Some(path) => format!(" {} {} — {} ▾ ", marker, side, path),
        None => format!(" {} {} — file browser ", marker, side),
    }
}

fn draw_file_switcher(f: &mut Frame, app: &mut App, body: Rect, theme: UiTheme) {
    let (panel, entries, selected, current_path) = {
        let Some(switcher) = app.file_switcher.as_ref() else {
            return;
        };
        (
            switcher.panel,
            switcher.entries.clone(),
            switcher.selected,
            app.panels[switcher.panel].path.clone(),
        )
    };
    let (left, _, right) = split_panels(body);
    let layout = LayoutGeometry { left, right };
    let Some(area) = file_switcher_area(layout, panel, entries.len()) else {
        return;
    };

    let items: Vec<ListItem> = entries
        .iter()
        .map(|e| {
            let current = current_path.as_ref().is_some_and(|p| p == &e.path);
            let label = if current {
                format!("{} ●", e.name)
            } else {
                e.name.clone()
            };
            ListItem::new(Line::from(Span::styled(
                label,
                Style::default().fg(theme.browser_file),
            )))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(selected));

    f.render_widget(Clear, area);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" switch file ")
                .border_style(
                    Style::default()
                        .fg(theme.border_focused)
                        .add_modifier(Modifier::BOLD),
                )
                .style(Style::default().bg(theme.help_bg)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.browser_highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, area, &mut state);

    if let Some(s) = app.file_switcher.as_mut() {
        s.selected = state.selected().unwrap_or(0);
    }
}

fn draw_file_content(f: &mut Frame, app: &mut App, idx: usize, area: Rect, theme: UiTheme) {
    let panel = &app.panels[idx];

    // Error reading the file: show the error inline.
    if let Some(err) = panel.error() {
        let line = Line::from(vec![
            Span::styled(
                "error: ",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(err.to_string()),
        ]);
        let para = Paragraph::new(line).wrap(Wrap { trim: false });
        f.render_widget(para, area);
        return;
    }

    let both = app.panels[LEFT].text().is_some() && app.panels[RIGHT].text().is_some();
    if both {
        let side = if idx == LEFT { &app.diff.left } else { &app.diff.right };
        render_diff_side(
            f,
            &app.diff,
            side,
            panel.highlighted.as_ref(),
            app.scroll,
            area,
            theme,
        );
    } else if let Some(text) = panel.text() {
        render_plain(f, text, panel.highlighted.as_ref(), app.scroll, area, theme);
    } else {
        // No file and no browser (shouldn't normally happen): show hint.
        let hint = Paragraph::new("press q is a no-op here — open a file from the other panel's browser")
            .style(Style::default().fg(theme.hint));
        f.render_widget(hint, area);
    }
}

fn render_diff_side(
    f: &mut Frame,
    diff: &SideBySide,
    side: &diff_tool_core::DiffSide,
    highlighted: Option<&Vec<Vec<Span<'static>>>>,
    scroll: usize,
    area: Rect,
    theme: UiTheme,
) {
    let line_no_width = count_digits(diff.len()) as u16;

    let mut lines: Vec<Line> = Vec::new();
    for row in side.rows.iter().skip(scroll) {
        let no = row
            .line_no
            .map(|n| format!("{:>width$}", n, width = line_no_width as usize))
            .unwrap_or_else(|| " ".repeat(line_no_width as usize));
        let no_span = Span::styled(no, Style::default().fg(theme.line_number));

        // Use cached syntax spans when the row maps to a real source line;
        // otherwise fall back to a single plain span.
        let mut spans: Vec<Span<'_>> = vec![no_span, Span::raw(" ")];
        match (row.line_no, highlighted) {
            (Some(n), Some(hl)) if n >= 1 && n <= hl.len() => {
                spans.extend(
                    hl[n - 1]
                        .iter()
                        .cloned()
                        .map(|s| span_with_diff_bg(s, row.kind, theme)),
                );
            }
            _ => {
                spans.push(Span::styled(
                    row.text.clone(),
                    diff_text_style(row.kind, theme),
                ));
            }
        }

        lines.push(Line::from(spans));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_plain(
    f: &mut Frame,
    text: &str,
    highlighted: Option<&Vec<Vec<Span<'static>>>>,
    scroll: usize,
    area: Rect,
    theme: UiTheme,
) {
    let line_no_width = count_digits(text.lines().count()) as u16;
    let mut lines: Vec<Line> = Vec::new();
    for (i, raw) in text.lines().enumerate().skip(scroll) {
        let no = format!("{:>width$}", i + 1, width = line_no_width as usize);
        let no_span = Span::styled(no, Style::default().fg(theme.line_number));
        let mut spans: Vec<Span<'_>> = vec![no_span, Span::raw(" ")];
        match highlighted {
            Some(hl) if i < hl.len() => spans.extend(hl[i].iter().cloned()),
            _ => spans.push(Span::raw(raw.to_string())),
        }
        lines.push(Line::from(spans));
    }
    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_browser(
    f: &mut Frame,
    app: &mut App,
    idx: usize,
    area: Rect,
    _focused: bool,
    theme: UiTheme,
) {
    // Borrow the browser out of the panel so we can also mutate list state.
    let show_path_input = app.path_input_active() && app.focused == idx;
    let path_input_text = if show_path_input {
        app.path_input.clone()
    } else {
        None
    };
    let panel = &mut app.panels[idx];
    let Some(browser) = panel.browser.as_mut() else {
        return;
    };

    let cwd_line = Line::from(vec![
        Span::styled("cwd: ", Style::default().fg(theme.browser_cwd_label)),
        Span::styled(
            browser.cwd.display().to_string(),
            Style::default().fg(theme.browser_cwd_path),
        ),
    ]);

    let path_input_line = show_path_input.then(|| {
        let input = path_input_text.as_deref().unwrap_or("");
        Line::from(vec![
            Span::styled("path: ", Style::default().fg(theme.browser_cwd_label)),
            Span::styled(input, Style::default().fg(theme.browser_cwd_path)),
            Span::styled("█", Style::default().fg(theme.browser_cwd_path)),
        ])
    });

    let top_constraints = if show_path_input {
        vec![
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ]
    } else {
        vec![Constraint::Length(1), Constraint::Min(0)]
    };
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints(top_constraints)
        .split(area);
    f.render_widget(Paragraph::new(cwd_line), inner[0]);

    let list_area = if show_path_input {
        f.render_widget(Paragraph::new(path_input_line.unwrap()), inner[1]);
        inner[2]
    } else {
        inner[1]
    };

    let items: Vec<ListItem> = browser
        .entries
        .iter()
        .map(|e| browser_item(e, theme))
        .collect();

    let mut state = ListState::default();
    state.select(Some(browser.selected));

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(theme.browser_highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, list_area, &mut state);

    // Sync the selection back in case ratatui would have changed it (it won't,
    // but this keeps the model authoritative if we later allow mouse drag).
    browser.selected = state.selected().unwrap_or(0);
}

fn browser_item(e: &Entry, theme: UiTheme) -> ListItem<'_> {
    let (symbol, style) = if e.is_dir {
        (
            "📁 ",
            Style::default()
                .fg(theme.browser_dir)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        ("📄 ", Style::default().fg(theme.browser_file))
    };
    let name = if e.is_dir {
        format!("{}{}/", symbol, e.name)
    } else {
        format!("{}{}", symbol, e.name)
    };
    ListItem::new(Line::from(Span::styled(name, style)))
}

/// Style for plain (non-syntax) diff text. Added/removed/changed rows use a
/// background highlight; equal/blank rows keep subtle foreground tints only.
fn diff_text_style(kind: RowKind, theme: UiTheme) -> Style {
    match theme.diff_bg(kind) {
        Some(bg) => Style::default().bg(bg),
        None => match kind {
            RowKind::Equal => Style::default().fg(theme.diff_equal),
            RowKind::Blank => Style::default().fg(theme.diff_blank),
            _ => Style::default(),
        },
    }
}

fn span_with_diff_bg(span: Span<'static>, kind: RowKind, theme: UiTheme) -> Span<'static> {
    let Some(bg) = theme.diff_bg(kind) else {
        return span;
    };
    Span::styled(
        span.content.to_string(),
        span.style.patch(Style::default().bg(bg)),
    )
}

fn draw_status(f: &mut Frame, app: &App, area: Rect, theme: UiTheme) {
    let stats = app.diff.stats();
    let focused_side = side_name(app.focused);

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled(
        format!(" focused: {} ", focused_side),
        Style::default()
            .fg(theme.status_focused)
            .add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("│ ", Style::default().fg(theme.status_separator)));

    let both = app.panels[LEFT].text().is_some() && app.panels[RIGHT].text().is_some();
    if both {
        spans.push(Span::styled(
            format!("+{} ", stats.added),
            Style::default().fg(theme.stat_added),
        ));
        spans.push(Span::styled(
            format!("-{} ", stats.removed),
            Style::default().fg(theme.stat_removed),
        ));
        spans.push(Span::styled(
            format!("~{} ", stats.changed),
            Style::default().fg(theme.stat_changed),
        ));
        spans.push(Span::styled(
            format!("={} ", stats.equal),
            Style::default().fg(theme.stat_equal),
        ));
    } else {
        spans.push(Span::styled("browsing ", Style::default().fg(theme.browsing)));
    }

    spans.push(Span::styled("│ ", Style::default().fg(theme.status_separator)));
    if app.file_switcher_active() {
        spans.push(Span::styled(
            "↑↓/j k: move  Enter/click: open  Esc: cancel",
            Style::default().fg(theme.hint),
        ));
    } else {
        spans.push(Span::styled(
            "click path: switch file  q: close file  Tab: switch panel  s: swap  t: theme  ?: help  Q/Ctrl-C: quit",
            Style::default().fg(theme.hint),
        ));
    }

    if let Some(msg) = &app.message {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(msg.clone(), Style::default().fg(theme.message)));
    }

    let line = Line::from(spans);
    let para = Paragraph::new(line).alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn draw_help(f: &mut Frame, area: Rect, theme: UiTheme) {
    let help = vec![
        "diff-tool — side-by-side file diff",
        "",
        "Mouse",
        "  click panel      focus a panel (or pick an entry in a browser)",
        "  click path title open sibling-file dropdown for that panel",
        "  click dropdown   select and open a file",
        "  scroll wheel     scroll the diff (or the dropdown)",
        "",
        "Diff view",
        "  j / ↓            scroll down      k / ↑            scroll up",
        "  J / PgDn         scroll 10         K / PgUp         scroll -10",
        "  g / Home         top               G / End          bottom",
        "  q                close focused panel's file → file browser",
        "  Tab              switch focused panel",
        "  s                swap left and right panels",
        "",
        "File switcher (path title dropdown)",
        "  click path title open sibling-file list",
        "  o                open sibling-file list (keyboard)",
        "  j / ↓  k / ↑     move selection",
        "  Enter / l / →    open selected file",
        "  Esc / q          cancel",
        "",
        "File browser",
        "  l / → / Enter    open file / enter directory",
        "  h / ← / Backsp   go to parent directory",
        "  H                toggle hidden files",
        "  /                type a path (Enter go, Esc cancel)",
        "  paste            jump to pasted file or directory",
        "  q                quit (when no file is open on the panel)",
        "",
        "Global",
        "  t                toggle dark / light theme",
        "  ?                toggle this help",
        "  Q  / Ctrl-C      quit the whole app",
    ];
    let lines: Vec<Line> = help
        .iter()
        .map(|s| Line::from(*s))
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help — press ? to close ")
        .style(Style::default().bg(theme.help_bg));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, centered(area, 70, 85));
}

fn centered(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let pop = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    let mid = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(pop[1]);
    mid[1]
}

fn count_digits(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        (n.ilog10() + 1) as usize
    }
}
