# AGENTS.md

## Cursor Cloud specific instructions

This is a Rust workspace (`diff-tool`) managed with **Pixi**. It produces one
product with two crates:

- `diff-tool-core` — shared library (side-by-side diff computation via the
  `similar` crate, plus a file browser model used by the TUI).
- `diff-tool` — the `diff-tool` binary crate: a ratatui TUI that shows two panels,
  a side-by-side diff between two files, mouse-click panel selection, a
  path-title file-switcher dropdown (click the path or press `o`; supports
  `../` and directory navigation), and a per-panel file browser (press `q` to
  close a panel's file; click entries to open). Each panel renders
  syntax-highlighted text via `syntect` (dark: base16-ocean; light: GitHub)
  with a custom `.log` grammar for log files. Press `c` to toggle compact mode
  (change hunks plus a few lines of context). Press `t` to toggle dark/light
  theme at runtime, or pass `--theme light` at startup. When `--theme` is
  omitted, the app probes the terminal background and picks a matching palette
  (works in VS Code, iTerm2, Ghostty, etc.).

### Toolchain (important)

- The system `cargo`/`rustc` may be too old (the dependency tree needs
  Rust ≥1.96). Running bare `cargo …` can fail with an `edition2024` error.
- Always go through Pixi, which provides the conda `rust` toolchain. Use
  `pixi run <task>` for the predefined tasks, or `pixi run -- cargo <args>`
  for ad-hoc cargo commands.

### Build / test / lint / run

Predefined tasks live in `pixi.toml`:

- Build: `pixi run build`
- Run TUI: `pixi run diff -- file_a.txt file_b.txt` (0, 1, or 2 file/dir args;
  a single file opens on both panels; a single directory opens a picker on both;
  missing paths open a picker at the nearest existing parent)
- Test: `pixi run test` (or `pixi run -- cargo test`)
- Lint: `pixi run clippy` (or `pixi run -- cargo clippy --release -- -D warnings`)

### Packaging

- Conda package: `pixi run conda-package` → `dist/diff-tool-*.conda`
- The rattler-build recipe lives in `recipe/recipe.yaml`.
- The binary is installed to `$PREFIX/bin/diff-tool`.

### Gotchas

- The two diff panels are row-aligned: `diff_lines` returns a `SideBySide`
  whose `left` and `right` sides always have the same length, so the TUI uses
  a single shared `scroll` offset for the diff view.
- `q` has panel-local semantics: it closes the focused panel's file and opens
  a browser; if the panel already has no file, `q` quits the app. Use
  `Q` / `Ctrl-C` to force-quit the whole app.

### Demo video

Do **not** regenerate or commit `demo/diff-tool-python-demo.mp4` in agent
changes (see `.cursor/rules/generate-demo-video.mdc`). The `pixi run demo-video`
task is for optional local use by humans only.

### CI

GitHub Actions (`.github/workflows/ci.yml`) runs `pixi run build`,
`pixi run test`, and `pixi run clippy` on pushes to `main` and on pull requests.