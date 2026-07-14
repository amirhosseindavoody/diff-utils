# Goals and limitations

## Product goal

**diff-tool** is a terminal UI for **comparing two text files side by side**,
with:

- Row-aligned added / removed / changed highlighting
- Syntax highlighting for common languages and log files
- Keyboard-first navigation (vim-style scroll keys) plus basic mouse support
- Per-panel file browsers so you can pick or swap files without restarting

It targets developers who want a fast, local, keyboard-friendly alternative to
opening a GUI diff tool or piping `diff` into a pager — especially when already
working in a terminal or over SSH.

## In scope

| Capability | Notes |
|------------|-------|
| Two-file side-by-side diff | Left = old/first file, right = new/second file |
| Line-level alignment | Equal, added, removed, changed, and blank padding rows |
| Syntax highlighting | syntect defaults + custom log grammar; dark (base16-ocean) or light (GitHub) theme |
| Interactive file picking | 0–2 CLI paths; browsers fill missing panels; path-title dropdown switches sibling files |
| Status summary | Counts of `+` added, `-` removed, `~` changed, `=` equal lines |
| Packaging | Pixi/conda global install and workspace-local dev |

## Out of scope (by design)

These are intentional boundaries, not missing features waiting for a quick patch:

| Limitation | Why |
|------------|-----|
| **No word- or character-level intra-line diff** | Line-aligned rows and shared scroll are core to the UI; see [engineering-decisions.md](engineering-decisions.md). |
| **No merge / patch / edit** | View-only comparison; no writing back to disk or 3-way merge. |
| **No directory or recursive diff** | Compares two files at a time; no `diff -r` equivalent. |
| **No Git integration** | No `git diff`, staging, or blame; pass file paths explicitly. |
| **No horizontal sync or column lock** | Long lines wrap within each panel; no synchronized horizontal scroll. |
| **Text files only** | Content is read as UTF-8 `String`; binary files are not supported meaningfully. |
| **Whole file in memory** | Very large files may be slow or exhaust memory; no streaming or mmap. |
| **Single shared vertical scroll in diff view** | Cannot scroll left and right panels independently in diff mode. |
| **Linux-first packaging** | Pixi workspace targets `linux-64`; other platforms may build from source but are not the primary packaged target. |

## Known behavioral constraints

- **Diff requires both panels loaded**: If only one file is open, the other panel
  shows its file or browser; no partial diff against empty content.
- **Changed hunks**: When a modified region has unequal numbers of deleted and
  inserted lines, pairing produces extra blank rows on the shorter side — still
  row-aligned, but not a minimal edit script.
- **Syntax detection**: By file extension and syntect heuristics; unknown
  extensions render as plain text without highlighting.
- **File browser**: Lists one directory at a time; paste or type a path to jump
  to a file or directory (`/` then type, or paste directly). No search,
  bookmarks, or multi-select.
- **Path-title file switcher**: Lists `../`, directories, and files in the
  current dropdown directory (hidden files omitted). Longer browsing sessions
  can still use the full panel browser.
- **Hidden files**: Off by default; toggle with `H` in browser mode.
- **Terminal requirements**: Needs a capable ANSI terminal, alternate screen,
  and raw mode; behavior in limited terminals may vary.

## Non-goals

- Replacing `git`, `delta`, `difftastic`, or IDE diff views for all workflows
- Serving as a general file manager
- Providing a scripting API beyond the CLI (the core crate is a library, but
  stability as a public API for third parties is not a stated goal)

## When to use something else

| Need | Better fit |
|------|------------|
| Word-level or structural diff | difftastic, meld, IDE diff |
| Patch application | `patch`, Git, dedicated merge tools |
| Directory trees | `diff -r`, `git diff`, fd + loop |
| Huge files (GB+) | Streaming diff tools or specialized comparators |
| Non-interactive CI output | `diff -u`, `git diff`, review UI in CI |
