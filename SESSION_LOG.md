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

## 2026-10-19 — Fix line-number gutter allocation

- The first row-layout adjustment was insufficient: the gutter only painted inside a child region, and painting does not advance egui's horizontal cursor. The editor therefore started at the gutter's x-position.
- The gutter now calls `allocate_space` on the explicitly sized left-to-right parent before painting. The editor is placed after the allocated gutter width, with no implicit column spacing.
- `cargo fmt` and `cargo check --offline --bin sublemon` passed. The user confirmed the GUI is fixed.

## 2026-10-19 — Reduce first display work for medium-sized source files

- The user reported opening `src/app.rs` took 12 seconds and requested under one second. At about 103 KiB, it was below the prior 512 KiB plain-text cutoff, so the first frame ran the fragmented token highlighter and wrapped the entire file.
- Moved the editor plain-text/no-wrap fast path to 64 KiB and prepared line metrics on the open worker for files above the same cutoff. Kept the LSP's independent 512 KiB analysis/synchronization cap.
- `cargo fmt`, `cargo check --offline --all-targets`, and `cargo build --offline --bin sublemon` passed. The 12-second scenario has not been timed again; confirm actual first-display latency before claiming the sub-second goal is achieved.

## 2026-10-19 — Repair intercepted text edits and prepare release packages

- Auto-pair and Enter indentation helpers directly edited the buffer before egui's normal `TextEdit` change callback. Their changes were not marked dirty, did not update metrics, and did not reach the language service. The caller now runs the shared change bookkeeping for these pre-edits, including the dirty revision, deferred metrics, and bounded LSP notification.
- Added Arch `PKGBUILD`, Debian and RPM package metadata, a desktop entry, and a GitHub Actions matrix-like workflow that produces native Arch, Debian, RPM, and Windows x86_64 artifacts.
- Local verification/build is in progress. Debian/RPM/Windows packaging cannot be performed locally with the installed tools; their native runner jobs are configured in CI.
- `gh auth status` reports the configured GitHub token is invalid. Push has not yet succeeded; retry with the repository's available SSH credentials or refresh GitHub CLI authentication before considering the release workflow triggered.

## 2026-10-19 — Inline completions and correct caret placement

- Replaced the large modal completion window with a muted inline continuation at egui's actual caret position. Tab accepts the selected completion, Escape dismisses it, and Up/Down change the candidate. Results are discarded if the text or caret moved while the LSP request was pending.
- Found the Enter caret bug: `prepare_enter_indentation` set the caret after the inserted newline, but the caller then moved it to the document end. Removed the redundant reset. The intercepted edit also runs normal dirty/revision, metrics, and LSP bookkeeping.
- Rust syntax coloring now remains active for `.rs` files up to 1 MiB. Other formats retain the 64 KiB plain-layout cutoff; all files above 64 KiB still disable wrapping.
- Official Sublime documentation search confirmed its built-in auto-complete is a caret-triggered completion popup with theme-defined rows and kind markers. This Sublemon implementation uses the user's requested inline ghost presentation, with independent styling.
- `cargo fmt` and `cargo check --offline --all-targets` pass. No tests were run. Release build is blocked waiting on a Cargo target lock left by an interrupted build; platform CI artifacts are not yet available.
- Remaining performance issue: egui TextEdit still lays out the whole document, and first-display latency was not measured after these changes. The 6-second report remains unresolved pending viewport-virtualized text rendering.
# Release orchestration

- Added executable `release.sh` to push the current branch and `v0.01`, wait for
  the GitHub Actions native build workflow, gather `.deb`, `.pkg.tar.zst`, `.rpm`,
  `.exe`, and `.zip` artifacts, and create a GitHub release with generated notes.
- Extended the Windows workflow artifact to include both `.exe` binaries as
  well as the portable zip. Added local state and collected release files to
  `.gitignore`.
- Did not run any package builds or publish a release in this environment.
- Validation: shell syntax check and diff review only.
