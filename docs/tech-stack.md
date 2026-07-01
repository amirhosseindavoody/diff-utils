# Tech stack

## Language and toolchain

| Item | Choice | Notes |
|------|--------|-------|
| Language | Rust (edition 2021) | Workspace crates share version and license via root `Cargo.toml` |
| Minimum Rust | ≥ 1.96 | Required by dependencies; use Pixi-provided `rust`, not system `cargo` |
| Package manager | [Pixi](https://pixi.sh/) | Conda-based env with pinned Rust toolchain and task definitions |
| License | MIT | See [LICENSE](../LICENSE) |

## Crates

### `diff-tool-core`

Shared library with no TUI dependencies. Keeps diff logic and the file browser
model testable without a terminal.

| Dependency | Role |
|------------|------|
| `similar` | Line-oriented text diff (`TextDiff::from_lines`) |
| `thiserror` | Typed errors for file browser I/O |

### `diff-tool`

Terminal binary: event loop, rendering, syntax highlighting.

| Dependency | Role |
|------------|------|
| `ratatui` | TUI layout and widgets |
| `crossterm` | Raw mode, alternate screen, keyboard and mouse events |
| `syntect` | Syntax highlighting (pure-Rust `regex-fancy`, no oniguruma) |
| `clap` | CLI parsing (0–2 optional file paths) |
| `anyhow` | Top-level error propagation in `main` |
| `diff-tool-core` | Diff computation and file browser |

## Build, test, and run

Predefined Pixi tasks (`pixi.toml`):

| Task | Command |
|------|---------|
| `build` | `cargo build --release` |
| `diff` | Run the TUI with optional file args |
| `test` | `cargo test` |
| `demo-video` | Regenerate demo via VHS (depends on `build`) |
| `conda-package` | `pixi publish --target-dir dist` |
| `update-version` | Bump version across manifests |

Ad-hoc Cargo commands:

```bash
pixi run -- cargo clippy --release
pixi run -- cargo test -p diff-tool-core
```

## Packaging

- **Conda / Pixi package**: `recipe/recipe.yaml` uses `pixi-build-rattler-build`.
  The release binary is installed to `$PREFIX/bin/diff-tool`.
- **Global install**: `pixi global install --git … diff-tool` (documented in README).
- **Platforms**: Pixi workspace currently targets `linux-64`; the Rust code itself
  is portable, but conda packaging is configured for Linux.

## Demo tooling

The demo video under `demo/` is produced with [VHS](https://github.com/charmbracelet/vhs),
`ffmpeg`, and `ttyd` (Pixi `demo` feature). This is development/marketing tooling,
not part of the runtime binary.

## Syntax highlighting details

- **Theme**: base16-ocean (fallback: base16-eighties, then first available default).
- **Built-in grammars**: syntect default syntax set (Python, Rust, JS, JSON, YAML,
  TOML, Markdown, C, etc.).
- **Custom grammar**: inline `.log` syntax for timestamps and log levels (`ERROR`,
  `WARN`, `INFO`, `DEBUG`, …).
