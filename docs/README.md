# diff-utils documentation

This folder describes how **diff-utils** is built, how it is structured, and the
trade-offs behind key design choices. For install, usage, and keyboard shortcuts,
see the [project README](../README.md).

| Document | Contents |
|----------|----------|
| [Tech stack](tech-stack.md) | Languages, libraries, build tooling, and packaging |
| [Architecture](architecture.md) | Crate layout, data flow, and component responsibilities |
| [Engineering decisions](engineering-decisions.md) | Rationale for major design choices |
| [Goals and limitations](goals-and-limitations.md) | What the tool is for, what it deliberately does not do |

## Quick reference

```
diff-utils-core/   shared library (diff + file browser model)
diff-utils/        ratatui TUI binary
recipe/            conda / pixi-build packaging
docs/              architecture and design notes (this folder)
```

Build and test through **Pixi** (not bare system `cargo` — see
[tech-stack.md](tech-stack.md)).
