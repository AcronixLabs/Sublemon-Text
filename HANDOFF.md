# Handoff: large-file responsiveness, built-in LSP, and welcome flow

## Current state

- The active native UI is eframe/egui (`src/app.rs`). `src/main.rs` launches the app normally. The language service is built as the small `sublemon-lsp` companion binary (`src/bin/sublemon-lsp.rs`); development runs that only build the GUI target fall back to launching the GUI executable with `--lsp-server`.
- Sublime-specific runtime compatibility has been removed from the active project: vendored syntax package data, generated grammar dump, package importer, package archive menu path, and Syntect dependencies are deleted. Syntax coloring is an original small token scanner in `src/syntax.rs`. Do not restore third-party editor packages or proprietary assets.
- The editor starts its own local LSP subprocess automatically whenever a workspace is open. The server is implemented in `src/lsp_server.rs`; its lexical features are completion, hover, definition/declaration/type-definition/implementation, references, document symbols, and unmatched-delimiter diagnostics. It does not invoke external language servers, provide compiler/type-checker semantics, or index unopened workspace files. Large documents over 512 KiB are not synchronized/analyzed.
- Large-file open reads, decodes, hashes, and initial line metric construction run on a worker. The saved-text duplicate was removed; large files skip syntax coloring and wrapping, line-metric refresh is deferred while typing, and the gutter paints visible rows only.
- App-wide recent project state is stored at `SUBLEMON_DATA_DIR/sublemon/state.json`, `$XDG_DATA_HOME/sublemon/state.json`, `%APPDATA%/sublemon/state.json`, or `$HOME/.local/share/sublemon/state.json` in that priority order. With no saved recent project the app shows Welcome. Once a folder is opened it is remembered; subsequent launches select the most recent existing folder and load its project roots. This newly introduced state file makes the first run after this change a welcome-screen run even if project-local `.sublemon.json` exists.
- Welcome includes Open Project, Open File, and recent project choices. Choosing a file from Welcome opens its parent folder as the project.

## Important remaining performance limitation

The large-file path reduces startup duplication and UI-side syntax/gutter work, but egui `TextEdit` still lays out the entire active buffer. Very large files can therefore remain slow during editing and scrolling. Do not report the performance issue fully resolved. The follow-up is a rope or piece-table document buffer with viewport-based line shaping/rendering, incremental line metrics, and edits applied without cloning/rebuilding the whole text. Preserve selection, undo, Unicode, IME, and save semantics when replacing the widget.

## Verification

- `cargo fmt` completed.
- `cargo check --offline --all-targets` completed successfully after the latest Rust edits, including the companion server target.
- No tests were run, in line with the workspace instruction not to run tests unless requested.
- The GUI was not launched in this session. Welcome appearance and startup/reopen behavior still need a manual desktop smoke check; the LSP protocol and large-file responsiveness also need runtime profiling.

## Next steps

1. Manually launch on a clean app-data directory and confirm the Welcome screen appears. Open a project, close the app, then confirm the next launch restores that most recent folder. Restore existing app data after any isolated smoke check.
2. Run representative 1 MiB, 10 MiB, and 100 MiB files through open, scroll, type, save, and close; measure frame latency and memory. Then implement a virtualized text surface because the current full-buffer `TextEdit` remains a bottleneck.
3. Exercise the built-in LSP over stdio with an editor client/protocol smoke check. Keep it lightweight and local; expand capability only with real syntax-aware parsing and bounded workspace indexing.
4. Update `PLAN.md`, `HANDOFF.md`, and `SESSION_LOG.md` after each subsequent work session.

## Worktree notes

This worktree contains broad, intentional deletions of the previously vendored syntax package tree and importer alongside source, dependency, documentation, welcome-state, and LSP edits. Preserve these changes. Do not reset or broadly restore the worktree. `.sublemon.json` is an existing untracked project settings file; inspect before editing it.
