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
- `resolve_path` / `navigate_target` — resolve a typed or pasted path (relative
  to `cwd`, tilde-expanded) as a file or directory target

The TUI decides when to show a browser (startup with no file, or after `q`
closes a panel's file).

### Path display (`path_display.rs`)

When both panels have files open, panel title bars compare the two paths and
replace shared leading and trailing path components with `...`, keeping only
the differing segments visible (for example `.../project-a/src/main.rs` vs
`.../project-b/src/main.rs`). When only one side has a file, that panel shows
the full path.

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
| `theme: UiTheme` | Dark or light UI palette and syntect theme name |
| `path_input: Option<String>` | Path typed or pasted in the focused panel's browser |

Each `Panel` has optional `path`, loaded `content`, optional `browser`, and
cached `highlighted` spans (per source line).

### Event loop

1. `crossterm`: raw mode, alternate screen, mouse capture.
2. Poll events (250 ms timeout); redraw on every iteration.
3. **Keyboard**: global keys (`?`, `Tab`, `s`, `t`, `q`, `Q`, Ctrl-C), then path
   input (when active), browser, or diff handlers depending on focused panel mode.
4. **Paste**: terminal paste events navigate to a file or directory in the focused
   panel's browser (or fill the path input when editing with `/`).
5. **Mouse**: click focuses left/right half by column; wheel scrolls diff; click
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

- Current directory path + optional path input line (`/` to type, paste to jump)
  + `List` of entries with selection highlight.

### Syntax highlighting (`highlight.rs`)

- Builds a `SyntaxSet` from syntect defaults plus embedded `LOG_SYNTAX`.
- Picks a syntect theme from the active `UiTheme` (base16-ocean for dark,
  GitHub for light).
- Highlights entire file sequentially (`HighlightLines`) so multi-line constructs
  stay correct.
- Converts syntect styles to ratatui `Span`s **without background**, so diff
  row highlights in `ui.rs` remain visible.

### UI theme (`theme.rs`)

- `UiTheme` holds ratatui colors for borders, status bar, file browser, help
  overlay, and diff row backgrounds.
- When `--theme` is omitted, `terminal.rs` probes the terminal background (OSC
  11) and selects dark or light automatically; pass `--theme dark|light` to
  override. Press `t` to toggle at runtime.
- Toggling theme refreshes cached syntax highlights for both panels.

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
diff-utils [--theme dark|light] [LEFT] [RIGHT]
```

- `--theme` — optional `dark` or `light`; when omitted, matches the terminal
  background (OSC 11 probe). Press `t` in the app to toggle.
- 0 args: both panels start in browser mode.
- 1 arg: left file loaded, right browser.
- 2 args: both files loaded, diff immediately.
