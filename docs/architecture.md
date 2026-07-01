# Architecture

## Overview

diff-utils is a two-crate Rust workspace: a **core library** for diff and file
browser logic, and a **TUI binary** that owns terminal I/O, rendering, and
syntax highlighting.

```
┌─────────────────────────────────────────────────────────────┐
│                      diff-utils (binary)                     │
│  main.rs ──► app.rs (state, events) ──► ui.rs (render)      │
│                    │                                         │
│                    └── highlight.rs (syntect → ratatui Span) │
└────────────────────────────┬────────────────────────────────┘
                             │ uses
┌────────────────────────────▼────────────────────────────────┐
│                   diff-utils-core (library)                  │
│  diff.rs          line diff, row alignment, stats            │
│  file_browser.rs  directory listing, navigation model        │
│  path_display.rs  abbreviated path titles for the TUI        │
└─────────────────────────────────────────────────────────────┘
```

## Core library (`diff-utils-core`)

### Side-by-side diff (`diff.rs`)

**Input**: two full file texts (`old`, `new`).

**Output**: `SideBySide { left: DiffSide, right: DiffSide }` where `left.len()
== right.len()` always. Each side is a vector of `DiffRow` with:

- `kind`: `Equal`, `Changed`, `Added`, `Removed`, or `Blank`
- `text`: line content (newlines stripped)
- `line_no`: 1-based source line number, or `None` for blank padding rows

**Pipeline**:

1. `similar::TextDiff::from_lines` produces equal / delete / insert changes.
2. Deletes pad the right side with `Blank`; inserts pad the left with `Blank`.
3. `realign_changes` collapses adjacent delete+insert runs into paired `Changed`
   rows so edits line up visually instead of appearing staggered.

There is **no intra-line (word-level) diff**; each row is one source line.

### File browser (`file_browser.rs`)

A minimal model for in-panel file picking:

- `FileBrowser::open` — root at a path or current working directory
- `refresh` — re-read directory; dirs first, then files, case-insensitive sort
- `go_up` / `enter_selected` — directory navigation
- `move_cursor` / `toggle_hidden` — selection and hidden-file filter

The TUI decides when to show a browser (startup with no file, or after `q`
closes a panel's file).

## TUI binary (`diff-utils`)

### Application state (`app.rs`)

`App` holds:

| Field | Purpose |
|-------|---------|
| `panels: [Panel; 2]` | Left and right halves |
| `focused` | Which panel receives keyboard input |
| `scroll` | **Single shared** vertical offset for the diff view |
| `diff: SideBySide` | Recomputed when either panel's file changes |
| `highlight: HighlightEngine` | Shared syntect state |

Each `Panel` has optional `path`, loaded `content`, optional `browser`,
cached `highlighted` spans (per source line), and `syntax_name` for the title bar.

### Event loop

1. `crossterm`: raw mode, alternate screen, mouse capture.
2. Poll events (250 ms timeout); redraw on every iteration.
3. **Keyboard**: global keys (`?`, `Tab`, `q`, `Q`, Ctrl-C), then browser or diff
   handlers depending on focused panel mode.
4. **Mouse**: click focuses left/right half by column; wheel scrolls diff; click
   in browser sets selection by row.

Terminal is restored on exit regardless of success or failure.

### Rendering (`ui.rs`)

Layout:

```
┌──────────────────┬─┬──────────────────┐
│  left panel      │ │  right panel     │
│  (header + body) │ │  (header + body) │
├──────────────────┴─┴──────────────────┤
│  status bar (+/-/~ stats, key hints)  │
└───────────────────────────────────────┘
```

**Diff mode** (both panels have readable files):

- Both sides render from the same `app.scroll` offset.
- Line numbers from `DiffRow.line_no`.
- Syntax spans come from pre-highlighted source lines indexed by `line_no`;
  diff row backgrounds (green/red/yellow) are patched on top.

**Single-file mode** (only one panel loaded):

- The loaded panel shows plain scrolled text with highlighting; no diff alignment.

**Browser mode**:

- Current directory path + `List` of entries with selection highlight.

### Syntax highlighting (`highlight.rs`)

- Builds a `SyntaxSet` from syntect defaults plus embedded `LOG_SYNTAX`.
- Highlights entire file sequentially (`HighlightLines`) so multi-line constructs
  stay correct.
- Converts syntect styles to ratatui `Span`s **without background**, so diff
  row highlights in `ui.rs` remain visible.

## Data flow when opening a file

```
User selects file in browser
        │
        ▼
Panel::load(path)          read file, clear browser
        │
        ▼
App::populate_highlight    syntect → Vec<Vec<Span>> per line
        │
        ▼
App::recompute_diff        diff_lines(left, right) if both sides ready
        │
        ▼
ui::draw                   scroll-aligned side-by-side view
```

## CLI

```text
diff-utils [LEFT] [RIGHT]
```

- 0 args: both panels start in browser mode.
- 1 arg: left file loaded, right browser.
- 2 args: both files loaded, diff immediately.
