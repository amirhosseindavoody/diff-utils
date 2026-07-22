# diff-tool documentation

This folder describes how **diff-tool** is built, how it is structured, and the
engineering choices behind it.

| Document | Contents |
|----------|----------|
| [Tech stack](tech-stack.md) | Rust toolchain, dependencies, Pixi tasks, packaging, CI |
| [Architecture](architecture.md) | Crate layout, data flow, rendering |
| [Engineering decisions](engineering-decisions.md) | Rationale for major choices |
| [Goals and limitations](goals-and-limitations.md) | Scope, non-goals, known limits |

## Layout (high level)

```
diff-tool-core/   shared library (diff + file browser model)
diff-tool/        ratatui TUI binary crate (`diff-tool` executable)
```
