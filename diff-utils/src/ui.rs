use crate::app::{side_name, App, Panel, LEFT, RIGHT};
use diff_utils_core::{Entry, RowKind, SideBySide};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

/// Top-level draw.
pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);
    let body = chunks[0];
    let status_bar = chunks[1];

    let (left, divider, right) = split_panels(body);

    draw_panel(f, app, LEFT, left);
    draw_divider(f, divider);
    draw_panel(f, app, RIGHT, right);
    draw_status(f, app, status_bar);

    if app.show_help {
        draw_help(f, area);
    }
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

fn draw_divider(f: &mut Frame, area: Rect) {
    let block = Block::default().borders(Borders::LEFT).border_style(Style::default().fg(Color::DarkGray));
    f.render_widget(block, area);
}

fn draw_panel(f: &mut Frame, app: &mut App, idx: usize, area: Rect) {
    let focused = app.focused == idx;
    let panel = &app.panels[idx];

    let title = panel_title(panel, idx, focused);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);
    let header_area = inner[0];
    let content_area = inner[1];

    let border_style = if focused {
        Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let header = Block::default()
        .borders(Borders::TOP)
        .title(title)
        .border_style(border_style);
    f.render_widget(header, header_area);

    if panel.browser.is_some() {
        draw_browser(f, app, idx, content_area, focused);
    } else {
        draw_file_content(f, app, idx, content_area);
    }
}

fn panel_title(panel: &Panel, idx: usize, focused: bool) -> String {
    let marker = if focused { "◀" } else { " " };
    let side = side_name(idx);
    match &panel.path {
        Some(p) => format!(" {} {} — {} ", marker, side, p.display()),
        None => format!(" {} {} — file browser ", marker, side),
    }
}

fn draw_file_content(f: &mut Frame, app: &mut App, idx: usize, area: Rect) {
    let panel = &app.panels[idx];

    // Error reading the file: show the error inline.
    if let Some(err) = panel.error() {
        let line = Line::from(vec![
            Span::styled("error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::raw(err.to_string()),
        ]);
        let para = Paragraph::new(line).wrap(Wrap { trim: false });
        f.render_widget(para, area);
        return;
    }

    let both = app.panels[LEFT].text().is_some() && app.panels[RIGHT].text().is_some();
    if both {
        let side = if idx == LEFT { &app.diff.left } else { &app.diff.right };
        render_diff_side(f, &app.diff, side, app.scroll, area);
    } else if let Some(text) = panel.text() {
        render_plain(f, text, app.scroll, area);
    } else {
        // No file and no browser (shouldn't normally happen): show hint.
        let hint = Paragraph::new("press q is a no-op here — open a file from the other panel's browser")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(hint, area);
    }
}

fn render_diff_side(
    f: &mut Frame,
    diff: &SideBySide,
    side: &diff_utils_core::DiffSide,
    scroll: usize,
    area: Rect,
) {
    let line_no_width = count_digits(diff.len()) as u16;

    let mut lines: Vec<Line> = Vec::new();
    for row in side.rows.iter().skip(scroll) {
        let no = row
            .line_no
            .map(|n| format!("{:>width$}", n, width = line_no_width as usize))
            .unwrap_or_else(|| " ".repeat(line_no_width as usize));
        let no_span = Span::styled(no, Style::default().fg(Color::DarkGray));
        let text_span = Span::styled(row.text.clone(), row_style(row.kind));
        let line = Line::from(vec![no_span, Span::raw(" "), text_span]);
        lines.push(line);
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn render_plain(f: &mut Frame, text: &str, scroll: usize, area: Rect) {
    let line_no_width = count_digits(text.lines().count()) as u16;
    let mut lines: Vec<Line> = Vec::new();
    for (i, raw) in text.lines().enumerate().skip(scroll) {
        let no = format!("{:>width$}", i + 1, width = line_no_width as usize);
        let no_span = Span::styled(no, Style::default().fg(Color::DarkGray));
        let line = Line::from(vec![no_span, Span::raw(" "), Span::raw(raw.to_string())]);
        lines.push(line);
    }
    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(para, area);
}

fn draw_browser(f: &mut Frame, app: &mut App, idx: usize, area: Rect, _focused: bool) {
    // Borrow the browser out of the panel so we can also mutate list state.
    let panel = &mut app.panels[idx];
    let Some(browser) = panel.browser.as_mut() else {
        return;
    };

    let cwd_line = Line::from(vec![
        Span::styled("cwd: ", Style::default().fg(Color::DarkGray)),
        Span::styled(browser.cwd.display().to_string(), Style::default().fg(Color::Yellow)),
    ]);
    // Show cwd on the first row of the content area, then list below it.
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    f.render_widget(Paragraph::new(cwd_line), inner[0]);

    let items: Vec<ListItem> = browser
        .entries
        .iter()
        .map(|e| browser_item(e))
        .collect();

    let mut state = ListState::default();
    state.select(Some(browser.selected));

    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, inner[1], &mut state);

    // Sync the selection back in case ratatui would have changed it (it won't,
    // but this keeps the model authoritative if we later allow mouse drag).
    browser.selected = state.selected().unwrap_or(0);
}

fn browser_item(e: &Entry) -> ListItem<'_> {
    let (symbol, style) = if e.is_dir {
        ("📁 ", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
    } else {
        ("📄 ", Style::default().fg(Color::White))
    };
    let name = if e.is_dir {
        format!("{}{}/", symbol, e.name)
    } else {
        format!("{}{}", symbol, e.name)
    };
    ListItem::new(Line::from(Span::styled(name, style)))
}

fn row_style(kind: RowKind) -> Style {
    match kind {
        RowKind::Equal => Style::default().fg(Color::Gray),
        RowKind::Added => Style::default().fg(Color::Green),
        RowKind::Removed => Style::default().fg(Color::Red),
        RowKind::Changed => Style::default().fg(Color::Yellow),
        RowKind::Blank => Style::default().fg(Color::DarkGray),
    }
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let stats = app.diff.stats();
    let focused_side = side_name(app.focused);

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled(
        format!(" focused: {} ", focused_side),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ));
    spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));

    let both = app.panels[LEFT].text().is_some() && app.panels[RIGHT].text().is_some();
    if both {
        spans.push(Span::styled(
            format!("+{} ", stats.added),
            Style::default().fg(Color::Green),
        ));
        spans.push(Span::styled(
            format!("-{} ", stats.removed),
            Style::default().fg(Color::Red),
        ));
        spans.push(Span::styled(
            format!("~{} ", stats.changed),
            Style::default().fg(Color::Yellow),
        ));
        spans.push(Span::styled(
            format!("={} ", stats.equal),
            Style::default().fg(Color::Gray),
        ));
    } else {
        spans.push(Span::styled(
            "browsing ",
            Style::default().fg(Color::Yellow),
        ));
    }

    spans.push(Span::styled("│ ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(
        "q: close file  Tab: switch panel  ?: help  Q/Ctrl-C: quit",
        Style::default().fg(Color::DarkGray),
    ));

    if let Some(msg) = &app.message {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(msg.clone(), Style::default().fg(Color::Magenta)));
    }

    let line = Line::from(spans);
    let para = Paragraph::new(line).alignment(Alignment::Left);
    f.render_widget(para, area);
}

fn draw_help(f: &mut Frame, area: Rect) {
    let help = vec![
        "diff-utils — side-by-side file diff",
        "",
        "Mouse",
        "  click           focus a panel (or pick an entry in a browser)",
        "  scroll wheel     scroll the diff",
        "",
        "Diff view",
        "  j / ↓            scroll down      k / ↑            scroll up",
        "  J / PgDn         scroll 10         K / PgUp         scroll -10",
        "  g / Home         top               G / End          bottom",
        "  q                close focused panel's file → file browser",
        "  Tab              switch focused panel",
        "",
        "File browser",
        "  l / → / Enter    open file / enter directory",
        "  h / ← / Backsp   go to parent directory",
        "  H                toggle hidden files",
        "  q                quit (when no file is open on the panel)",
        "",
        "Global",
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
        .style(Style::default().bg(Color::Black));
    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(para, centered(area, 60, 70));
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
