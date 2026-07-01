# diff-utils

A side-by-side **file diff TUI** written in Rust — two panels, one diff, mouse
and keyboard driven.

## Demo

Side-by-side diff of two Python files with syntax highlighting and added /
removed / changed row backgrounds:

<p align="center">
  <video src="https://github.com/amirhosseindavoody/diff-utils/raw/main/demo/diff-utils-python-demo.mp4" controls playsinline width="900">
    <a href="demo/diff-utils-python-demo.mp4">Download demo video</a>
  </video>
</p>

<sub>Recorded with <a href="https://github.com/charmbracelet/vhs">VHS</a> — regenerate via <code>pixi run demo-video</code>.</sub>

## Install

Install globally with pixi (adds `diff-utils` to your PATH):

```bash
pixi global install --git https://github.com/amirhosseindavoody/diff-utils.git --branch main diff-utils
```

Then run:

```bash
diff-utils file_a.txt file_b.txt
```

## Features

| Surface | What it does |
|---------|--------------|
| **Two-panel diff** | Left panel = file A, right panel = file B, with added / removed / changed lines highlighted and aligned row-for-row. |
| **Syntax highlighting** | Per-panel syntax highlighting via `syntect` (dark: base16-ocean; light: Solarized). Common languages work out of the box — Python, Rust, JS, JSON, YAML, TOML, Markdown, C, and more — plus a custom `.log` syntax that colors timestamps and `ERROR`/`WARN`/`INFO`/`DEBUG` levels. Diff backgrounds (added/removed/changed) are preserved under the syntax colors. |
| **Dark / light theme** | Default dark UI; pass `--theme light` or press `t` in the app to switch to a light palette (syntax colors and diff backgrounds update immediately). |
| **Mouse selection** | Click either panel to focus it. Scroll wheel scrolls the diff. |
| **Per-panel file browser** | Press `q` on a panel to close its file and open an in-panel file browser to pick a different file for that panel. Paste a path or press `/` to jump directly to a file or directory. |

## Quick start

### Prerequisites

- [Pixi](https://pixi.sh/latest/)

### From source

```bash
git clone https://github.com/amirhosseindavoody/diff-utils.git
cd diff-utils
pixi install
pixi run build
pixi run diff -- file_a.txt file_b.txt
```

Launch with zero, one, or two files — any panel without a file opens a file
browser:

```bash
pixi run diff --                       # both panels start as browsers
pixi run diff -- file_a.txt            # right panel starts as a browser
pixi run diff -- file_a.txt file_b.txt # straight into the diff
```

### Install with pixi (another workspace)

Enable git source builds, then add from GitHub:

```toml
# pixi.toml
[workspace]
preview = ["pixi-build"]
```

```bash
pixi add --git https://github.com/amirhosseindavoody/diff-utils.git --branch main diff-utils
```

After install, `diff-utils` is available in the pixi environment.

Install globally (adds `diff-utils` to your PATH):

```bash
pixi global install --git https://github.com/amirhosseindavoody/diff-utils.git --branch main diff-utils
```

## Usage

### Diff view

```bash
pixi run diff -- old.txt new.txt
pixi run diff -- --theme light old.txt new.txt
```

- **Mouse click** — focus a panel
- **Mouse wheel** — scroll the diff
- `j` / `↓`  scroll down        `k` / `↑`  scroll up
- `J` / `PgDn`  scroll 10        `K` / `PgUp`  scroll -10
- `g` / `Home`  top              `G` / `End`  bottom
- `q`  close the focused panel's file → file browser
- `Tab`  switch focused panel
- `s`  swap left and right panels
- `t`  toggle dark / light theme
- `?`  toggle help

### File browser (per panel)

Pressing `q` on a panel closes that panel's file and shows a file browser
rooted at the file's parent directory, so you can quickly swap to a sibling
file.

- `l` / `→` / `Enter`  open file / enter directory
- `h` / `←` / `Backspace`  go to parent directory
- `H`  toggle hidden files
- `/`  type a path (`Enter` go, `Esc` cancel)
- paste  jump to pasted file or directory path
- `q`  quit the app (when the panel has no file open)

### Direct cargo (from repo root)

Pixi provides Rust ≥ 1.96; bare system `cargo` may be too old:

```bash
pixi run build
./target/release/diff-utils old.txt new.txt
```

## Testing

```bash
pixi run -- cargo test
```

## Conda package

Build a `.conda` package (includes the `diff-utils` binary):

```bash
pixi run conda-package
```

Artifact: `dist/diff-utils-*.conda`.

## Documentation

Design notes, architecture, and engineering decisions live in [`docs/`](docs/):

- [Tech stack](docs/tech-stack.md) — Rust toolchain, dependencies, Pixi tasks, packaging
- [Architecture](docs/architecture.md) — crate layout, data flow, rendering
- [Engineering decisions](docs/engineering-decisions.md) — rationale for major choices
- [Goals and limitations](docs/goals-and-limitations.md) — scope, non-goals, known limits

## Project structure

```
diff-utils-core/  # shared library (side-by-side diff + file browser model)
diff-utils/       # `diff-utils` binary (ratatui TUI)
docs/             # architecture and design documentation
recipe/           # rattler-build recipe for pixi/conda packaging
```

## License

MIT — see [LICENSE](LICENSE).
