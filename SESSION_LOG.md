# Session log

## 2026-10-14 — Move the native UI from egui to Floem

- Replaced the active eframe/egui dependency with Floem 0.2 and launched a new Floem frontend from `src/main.rs`.
- Added `src/floem_app.rs` with the native window/icon, document tabs, virtualized file list, open/save/folder/package actions, Markdown preview, theme toggle, and keyboard shortcuts.
- Added `src/floem_syntax.rs` to apply bundled Syntect syntax definitions and Sublemon palettes through Floem's editor styling interface.
- Kept existing in-progress egui adapter and LSP files in the worktree but excluded them from the active build, preserving user changes for later reconciliation.
- Updated `AGENTS.md`, `PLAN.md`, `README.md`, and third-party notices to reflect Floem and the current migration gaps.
- Ran `cargo fmt` and `cargo check --offline`; both completed successfully. No tests were run and the GUI was not launched in this session.
- Remaining active-UI gaps include LSP, menu command parity, persisted workspace/multi-root state, dirty-close confirmation, and background project scanning.

## 2026-10-14 — Prepare pixel-fidelity handoff

- Reworked `HANDOFF.md` into a detailed next-session task focused on restoring screenshot-level UI fidelity in the active Floem frontend.
- Documented the 941 × 1038 reference dimensions, screenshot composition, stale pre-Floem baseline, relevant source/assets/tools, staged implementation order, and visual verification target.
- Recorded worktree preservation and remaining Floem migration caveats for the next agent.

## 2026-10-14 — Fix Floem reactive borrow panic

- Moved active-tab signal updates out of mutable `tabs.update` closures in new-tab creation and file opening. Floem may rerun subscribers synchronously during signal updates, so writing `active` while `tabs` remains mutably borrowed could panic when subscribers read the tab list.
- Ran `cargo fmt` and `cargo check --offline`; both succeeded with only existing dead-code warnings.
- Launched `cargo run --bin sublemon` in the desktop session after the change; it remained running without the reported panic. Stopped the app after startup verification. Did not exercise UI controls or run tests.

## 2026-10-17 — Disable Rust Analyzer autostart

- Removed the `rust-analyzer` command from the project `.sublemon.json` settings and removed automatic rust-analyzer selection for Rust files.
- Kept generic LSP support and other language-server defaults; an explicitly configured server command can still be used. Saving project settings with no command now removes the command field instead of persisting an empty value.
- Updated README usage and HANDOFF state.
- `cargo fmt` and `cargo check --offline` passed. No rust-analyzer process was running during the check.

## 2026-10-19 — Large-file path, built-in language service, and welcome startup

- Removed the bundled Sublime syntax packages, generated syntax dump, archive importer, archive menu path, and Syntect/ZIP dependencies. Replaced grammar-based styling with an original lightweight token scanner and updated dependency notices and clean-room instructions.
- Added a local stdio JSON-RPC LSP server in `src/lsp_server.rs`, launched automatically as a Sublemon subprocess for open workspaces. It provides lexical completion, hover, local navigation/references, document symbols, and unmatched-delimiter diagnostics. It skips open documents larger than 512 KiB and does not start external language servers.
- Added a standalone `sublemon-lsp` binary so normal builds can run the protocol service without starting a second GUI process; development falls back to the GUI executable if the companion has not been built.
- Reduced large-file open/render overhead: open/decode/hash and initial metrics now happen on a worker; removed the duplicate saved-text copy and duplicate file read; large files skip syntax coloring and wrapping; gutter rendering uses visible rows; line metric rebuilds are deferred briefly after edits.
- Added global recent-project persistence and a Welcome screen. The new state file has no prior history, so first startup after this change shows Welcome; opening a folder remembers it, and a later startup selects the latest existing project.
- Replaced the stale Floem migration handoff with current architecture, limitations, verification, and next steps. Updated README, PLAN, AGENTS, and third-party notices to reflect the removed package ecosystem and in-tree language service.
- `cargo fmt` and `cargo check --offline` passed. No tests were run and the GUI was not launched.
- Remaining issue: egui `TextEdit` shapes the entire active buffer, so very large files may still be slow during editing/scrolling. A viewport-rendered rope/piece-table editor and runtime profiling remain necessary before claiming large-file performance is fixed.
