# Engineering decisions

This document records **why** the codebase is shaped the way it is. When you
change behavior, update this file if a decision is reversed or a new trade-off
is introduced.

## Two-crate split: core vs TUI

**Decision**: Put diff computation and the file browser in `diff-tool-core`;
keep ratatui, crossterm, and syntect in `diff-tool`.

**Rationale**:

- Core logic is unit-tested without spinning up a terminal (`diff.rs` tests).
- Dependencies stay minimal in the library (no GUI stack).
- A future non-TUI front end (e.g. web or CLI plain output) could reuse the core.

## Line-level diff only

**Decision**: Compare whole lines; do not highlight changed words within a line.

**Rationale**:

- Row alignment is the central UI model: left and right must always have the
  same number of rows with a shared scroll index.
- Word-level diff would require a different rendering model (inline spans within
  one row) and more complexity for marginal gain in a file-oriented tool.
- Line diff via `similar` is fast and predictable on large files.

## `realign_changes` for modified regions

**Decision**: After `similar` emits delete-then-insert runs for a changed hunk,
pair them into `RowKind::Changed` on both sides.

**Rationale**:

- Raw delete/insert alignment shows staggered blanks (removed on left only, then
  added on right only), which is hard to scan.
- Pairing by position within the hunk keeps edits on the same visual row when
  line counts match; unmatched lines remain removed-only or added-only with blanks.

## Single shared scroll offset

**Decision**: One `app.scroll` drives both diff panels.

**Rationale**:

- `SideBySide` guarantees equal row counts; scrolling must stay synchronized so
  row N on the left always corresponds to row N on the right.
- Separate scroll positions would desync the side-by-side view.

## Per-panel file browser and panel-local `q`

**Decision**: Each panel can independently hold a file or a browser. `q` closes
the focused panel's file and opens a browser rooted at that file's parent; if
the panel already has no file, `q` quits the app.

**Rationale**:

- Supports swapping one side's file without disturbing the other (common when
  comparing siblings in the same directory).
- Parent-directory root makes sibling selection fast after closing a file.
- `Q` / Ctrl-C remain explicit whole-app quit.

## Syntax highlight cache keyed by source line

**Decision**: Highlight full files once into `Vec<Vec<Span>>` indexed by source
line number; at render time, map diff rows via `line_no` and apply diff
backgrounds on top.

**Rationale**:

- Re-highlighting on every frame would be expensive.
- Diff rows reference source lines (or blanks); indexing by line number keeps
  highlight correct for equal and changed rows.
- Syntect backgrounds are stripped so diff row colors are not obscured.

## syntect with `regex-fancy` (no oniguruma)

**Decision**: Disable syntect default features; enable `regex-fancy` only.

**Rationale**:

- Avoids linking the oniguruma C library, which complicates conda/pixi builds
  and cross-platform packaging.
- Pure-Rust regex is sufficient for bundled syntax definitions.

## Custom `.log` grammar

**Decision**: Register a small inline YAML syntax for `.log` / `.syslog` / `.out`
extensions.

**Rationale**:

- Log files are common in diff workflows but not a first-class syntect default
  with useful level/timestamp coloring for this tool's audience.
- Inline definition avoids shipping extra asset files.

## Dark and light UI themes

**Decision**: Centralize ratatui colors in `theme.rs`; pair each scheme with a
matching syntect theme (base16-ocean for dark, GitHub for light). When
`--theme` is omitted, probe the terminal background (OSC 11) and pick a matching
scheme. Expose explicit `--theme dark|light` and `t` to toggle at runtime.

**Rationale**:

- Diff backgrounds and chrome colors were hard-coded for dark terminals; a light
  palette needs softer pastels and darker foreground accents.
- VS Code and other integrated terminals often use a light background while the
  app previously defaulted to dark — syntax and diff colors were unreadable.
- Syntax highlighting must switch with the UI so contrast stays readable.
- Re-highlighting on toggle is acceptable because theme changes are infrequent.

## Force color in the TUI

**Decision**: Call `crossterm::style::force_color_output(true)` at startup.

**Rationale**:

- CI and cloud shells often set `NO_COLOR`; the product is unusable without
  diff backgrounds and syntax colors in a full-screen TUI.

## Pixi as the canonical toolchain

**Decision**: Document and CI-oriented workflows go through `pixi run …`, not
bare `cargo`.

**Rationale**:

- System Rust on many machines is older than 1.96; dependency tree uses features
  that require a recent compiler.
- Pixi also unifies conda packaging, demo tooling, and Rust in one manifest.

## Full file load in memory

**Decision**: Read entire files into `String` on load.

**Rationale**:

- Simplicity: diff and highlight both need full text; streaming would require
  a different architecture.
- Acceptable for the intended use (typical source and log files), with known
  limits documented in [goals-and-limitations.md](goals-and-limitations.md).

## Mouse support without full mouse-driven diff navigation

**Decision**: Mouse focuses panels, scrolls the diff, selects browser rows, and
opens a sibling-file dropdown from the path title; keyboard remains primary for
scrolling and most file picking (`o` mirrors the path-title click).

**Rationale**:

- Keeps event handling small while supporting the most common mouse actions in
  a terminal diff viewer.
- Path-title switching covers the frequent “swap to a sibling file” case without
  forcing a full browser round-trip (`q` → navigate → open).

## Path-title sibling-file dropdown

**Decision**: Clicking a panel's file-path title (or pressing `o`) lists only
files in that file's directory — not directories — in an overlay dropdown.

**Rationale**:

- Sibling swap is the common follow-up after opening a diff; a lightweight
  dropdown is faster than closing into the full browser.
- Directories stay out of the list so every entry is immediately openable as a
  file; use the full browser (`q`) for directory navigation.
