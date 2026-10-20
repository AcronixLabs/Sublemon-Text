# Handoff: large-file responsiveness, built-in LSP, and welcome flow

## Current state

- The active native UI is eframe/egui (`src/app.rs`). `src/main.rs` launches the app normally. The language service is built as the small `sublemon-lsp` companion binary (`src/bin/sublemon-lsp.rs`); development runs that only build the GUI target fall back to launching the GUI executable with `--lsp-server`.
- Sublime-specific runtime compatibility has been removed from the active project: vendored syntax package data, generated grammar dump, package importer, package archive menu path, and Syntect dependencies are deleted. Syntax coloring is an original small token scanner in `src/syntax.rs`. Do not restore third-party editor packages or proprietary assets.
- The editor starts its own local LSP subprocess automatically whenever a workspace is open. The server is implemented in `src/lsp_server.rs`; its lexical features are completion, hover, definition/declaration/type-definition/implementation, references, document symbols, and unmatched-delimiter diagnostics. It does not invoke external language servers, provide compiler/type-checker semantics, or index unopened workspace files. Large documents over 512 KiB are not synchronized/analyzed.
- Open reads, decodes, hashes, and initial line metric construction run on a worker. The saved-text duplicate was removed; files over 64 KiB disable wrapping, Rust coloring remains through 1 MiB, line-metric refresh is deferred while typing, and the gutter paints visible rows only. Other file types skip syntax coloring above 64 KiB. The service independently skips files over 512 KiB.
- App-wide recent project state is stored at `SUBLEMON_DATA_DIR/sublemon/state.json`, `$XDG_DATA_HOME/sublemon/state.json`, `%APPDATA%/sublemon/state.json`, or `$HOME/.local/share/sublemon/state.json` in that priority order. With no saved recent project the app shows Welcome. Once a folder is opened it is remembered; subsequent launches select the most recent existing folder and load its project roots. This newly introduced state file makes the first run after this change a welcome-screen run even if project-local `.sublemon.json` exists.
- Welcome includes Open Project, Open File, and recent project choices. Choosing a file from Welcome opens its parent folder as the project.

## Important remaining performance limitation

The >64 KiB fast path avoids wrapping that delayed first display of `src/app.rs`; Rust syntax coloring now stays active to 1 MiB. egui `TextEdit` still lays out the entire active buffer. Very large files can therefore remain slow during editing and scrolling. First-open time for `src/app.rs` has not yet been measured after this change. Do not claim the sub-second target verified without timing it. The follow-up is a rope or piece-table document buffer with viewport-based line shaping/rendering, incremental line metrics, and edits applied without cloning/rebuilding the whole text. Preserve selection, undo, Unicode, IME, and save semantics when replacing the widget.

## Verification

- `cargo fmt` completed.
- `cargo check --offline --all-targets` completed successfully after the latest Rust edits, including the companion server target.
- No tests were run, in line with the workspace instruction not to run tests unless requested.
- The GUI was not launched in this session. Welcome appearance and startup/reopen behavior still need a manual desktop smoke check; the LSP protocol and large-file responsiveness also need runtime profiling.

## Latest gutter overlap fix

The gutter and editor now sit in an explicitly sized left-to-right row with zero implicit spacing. Crucially, the gutter calls `allocate_space` in the parent row before painting: painting alone did not advance egui's horizontal cursor, so the editor started at the gutter's x-position. The user confirmed the GUI is fixed.

## Latest file-open performance change

`src/app.rs` is about 103 KiB, which fell below the old 512 KiB syntax cutoff and forced the per-token highlighter to build a very fragmented layout. The plain-text, no-wrap fast path now starts above 64 KiB; line metrics for these files are also prepared on the file-open worker. The LSP analysis cap remains 512 KiB. `cargo fmt`, `cargo check --offline --all-targets`, and `cargo build --offline --bin sublemon` pass. The 12-second open was not timed again after this change, so the requested sub-second target still needs runtime measurement. egui still lays out the full active buffer.

## Current task: editing and platform releases

The auto-pair and Enter handlers mutate the document before `TextEdit::show`. Their change notifications were previously ignored, which could make typed characters appear to vanish or fail to persist/synchronize. `pre_edit_changed` now runs dirty revision, deferred metrics, and bounded LSP change bookkeeping. Review this path and manually type quotes, brackets, and Enter in the GUI if a desktop session is available.

Packaging metadata is present for Arch (`PKGBUILD`), Debian/RPM (`Cargo.toml` metadata), plus native GitHub Actions builds for Arch, Debian, RPM, and Windows x86_64. A push should trigger artifact builds. Local `gh auth status` currently reports an invalid token, and Docker daemon access is denied, so remote authentication/build status remains unresolved. Do not claim built packages until the workflow completes and artifacts are inspected.

## Current UI follow-up: inline completions and editor input

The user asked to replace the large language completion window with inline suggestions. The current implementation paints the selected completion's remaining text beside the actual egui caret in subdued text; Tab accepts, Escape dismisses, and arrows cycle. Stale results are rejected when text or caret position changed. Sublime's public docs describe a caret-triggered popup; this custom ghost-text treatment follows the user's explicit inline request and uses no proprietary assets. `TextEditOutput.galley` and `galley_pos` provide the caret anchor.

The Enter helper already placed the caret after its inserted newline and indentation, but its caller then reset the caret to the end of the entire file. That reset is removed. Rust files retain lexical coloring through 1 MiB, while the no-wrap large-file editor mode remains enabled above 64 KiB. `cargo check --offline --all-targets` and `cargo fmt --check` pass after these changes. Full release build is still pending because Cargo reported waiting for a lock held by an interrupted build.

Remaining performance limitation: egui's built-in multiline `TextEdit` still lays out the whole buffer, even with no wrapping and syntax-light styling. The 6-second open complaint is not yet resolved to the <1-second target. A viewport-rendered rope/piece-table editor is still the proper architectural fix. Continue from the current inline completion and caret placement code and perform a desktop typing smoke check if the GUI is available.

## Next steps

1. Manually launch on a clean app-data directory and confirm the Welcome screen appears. Open a project, close the app, then confirm the next launch restores that most recent folder. Restore existing app data after any isolated smoke check.
2. Run representative 1 MiB, 10 MiB, and 100 MiB files through open, scroll, type, save, and close; measure frame latency and memory. Then implement a virtualized text surface because the current full-buffer `TextEdit` remains a bottleneck.
3. Exercise the built-in LSP over stdio with an editor client/protocol smoke check. Keep it lightweight and local; expand capability only with real syntax-aware parsing and bounded workspace indexing.
4. Update `PLAN.md`, `HANDOFF.md`, and `SESSION_LOG.md` after each subsequent work session.

## Worktree notes

This worktree contains broad, intentional deletions of the previously vendored syntax package tree and importer alongside source, dependency, documentation, welcome-state, and LSP edits. Preserve these changes. Do not reset or broadly restore the worktree. `.sublemon.json` is an existing untracked project settings file; inspect before editing it.
# Release script

- `release.sh` pushes the current clean branch and `v0.01` (override with
  `RELEASE_VERSION=...`) over the configured HTTPS Git remote, then waits for
  the public GitHub release page to appear. It performs no local builds.
- The tag-triggered `publish` job creates the GitHub release with its
  repository token after the native Arch, Debian, RPM, and Windows builds pass.
- Requires HTTPS Git push access, `curl`, `python3`, an `origin` remote, and a
  committed clean worktree. It refuses existing local or remote release tags.
- The crate and package metadata still say `0.1.0`; synchronize those with
  release `0.01` before publishing if package-manager version consistency is
  required.

## macOS DMG and Save picker

- `.github/workflows/release-builds.yml` now has a macOS 15 job that cross-builds universal Apple Silicon and Intel binaries, assembles an unsigned `.app`, and creates a compressed unsigned DMG. The artifact is uploaded by CI and included in tagged releases. It has not yet been exercised on a macOS runner.
- Save and Save As now open the native save picker. The picker starts in the current file's parent directory and preselects its name; Ctrl+S/Command+S and the close-confirmation Save action use it too. Selecting the current path uses the document's normal external-change guard.
- No local macOS/DMG build is possible on this Linux host; rely on the hosted macOS runner and inspect its uploaded artifact before claiming success.
