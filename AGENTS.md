# AGENTS.md

## Cursor Cloud specific instructions

This is a Rust workspace (`diff-utils`) managed with **Pixi**. It produces one
product with two crates:

- `diff-utils-core` — shared library (side-by-side diff computation via the
  `similar` crate, plus a file browser model used by the TUI).
- `diff-utils` — the `diff-utils` binary: a ratatui TUI that shows two panels,
  a side-by-side diff between two files, mouse-click panel selection, and a
  per-panel file browser (press `q` to close a panel's file).

### Toolchain (important)

- The system `cargo`/`rustc` may be too old (the dependency tree needs
  Rust ≥1.96). Running bare `cargo …` can fail with an `edition2024` error.
- Always go through Pixi, which provides the conda `rust` toolchain. Use
  `pixi run <task>` for the predefined tasks, or `pixi run -- cargo <args>`
  for ad-hoc cargo commands.

### Build / test / lint / run

Predefined tasks live in `pixi.toml`:

- Build: `pixi run build`
- Run TUI: `pixi run diff -- file_a.txt file_b.txt` (0, 1, or 2 file args)
- Test: `pixi run test` (or `pixi run -- cargo test`)
- Lint: `pixi run -- cargo clippy --release`

### Packaging

- Conda package: `pixi run conda-package` → `dist/diff-utils-*.conda`
- The rattler-build recipe lives in `recipe/recipe.yaml`.
- The binary is installed to `$PREFIX/bin/diff-utils`.

### Gotchas

- The two diff panels are row-aligned: `diff_lines` returns a `SideBySide`
  whose `left` and `right` sides always have the same length, so the TUI uses
  a single shared `scroll` offset for the diff view.
- `q` has panel-local semantics: it closes the focused panel's file and opens
  a browser; if the panel already has no file, `q` quits the app. Use
  `Q` / `Ctrl-C` to force-quit the whole app.
