# Handoff

## Latest completed step — themes, native pickers, performance plan

- Added a **View → Light Theme** toggle with light and dark chrome, editor backgrounds, line gutter, selection, and syntax palettes. Dark remains the startup default; theme persistence is not implemented.
- Reworked **File → Open → File… / Folder…** to use `rfd` native pickers. File opens the selected document; folder replaces the project root. Existing Project → Add Folder remains for adding another root.
- Added `rfd` 0.17.2 and recorded its MIT license in `THIRD_PARTY_NOTICES.md`.
- Expanded PLAN step 10 into a profiling-led performance plan with reproducible document sizes, initial latency/idle targets, frame-path optimization, and a benchmark gate for rope/piece-table work. GPU drawing will not accelerate parsing, I/O, or allocations.
- `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. GUI, platform-native picker behavior, and performance were not manually tested/profiled.

## Latest completed step — renderer and editor allocation reduction

- Selected eframe's `wgpu` renderer explicitly; eframe already had the wgpu feature enabled. GPU adapter availability and performance are platform-dependent and were not manually measured.
- Replaced the per-frame full document clone used for gutter rendering with a compact vector of line character counts. Markdown preview precomputes display rows only while preview is active.
- `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. No GUI profiling or GPU adapter verification was performed.

## Latest completed step — auto-pairs and Markdown preview

- Added auto-closing `()`, `[]`, `{}`, single/double quotes, and backticks. Selected text is wrapped; typing an existing matching closer skips over it.
- Added `Ctrl+Shift+V` to toggle a scrollable, lightweight Markdown preview with heading, list, quote, and fenced-code marker styling.
- `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. GUI behavior has not been manually exercised. Preview is intentionally basic and does not yet render inline Markdown emphasis/links or full CommonMark.

## Latest completed step — shipped syntax packages

- Vendored all 48 package folders from `sublimehq/Packages` snapshot `f3ab97b3a1440ecf2645dd287f37c7e9807c1d45` under `assets/sublime-syntax-packages/`, retaining upstream and file-level license notices.
- The checked-in compiled data is `assets/sublemon-syntaxes.packdump`. The independently authored Syntect renderer selects syntax by file path and applies Sublemon colors. 78/202 upstream syntax definitions are accepted directly; others use Syntect's default grammar when available, otherwise plain text.
- Rebuild the syntax data after updating source definitions with `cargo run --bin rebuild_syntax_data`.
- The linked repository contains syntax packages, not the LSP client. Sublemon's LSP code remains independently implemented; Sublime's separate LSP repository is not included, and imported package code is not executed.
- `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. No GUI or language-by-language highlighting review was performed.

## Current project state

- The `sublimehq/Packages` snapshot is vendored under `assets/sublime-syntax-packages/` (commit recorded in `UPSTREAM_REVISION`) with its repository license and file-level notices. It contains syntax packages, not the LSP client.
- Syntax highlighting uses a compiled Syntect database at `assets/sublemon-syntaxes.packdump` with an independently authored Sublemon palette. The snapshot has 202 syntax definitions; Syntect parses 78 directly. The other files remain bundled, with Syntect's default grammar fallback where it matches or plain text otherwise. After grammar-data changes regenerate with `cargo run --bin rebuild_syntax_data`.
- Sublemon does not execute package plugins. Its LSP client remains independently implemented; Sublime's separate LSP repository is not vendored.
- The editor and line-number gutter now share a vertically scrolling viewport whose content height tracks the document's visual rows. This addresses the prior issue where the multiline editor was placed in a fixed-height allocation without an enclosing scroll area.
- The project tree uses a virtualized scroll area and the Material Icon Theme v5.39.0 assets/associations. Attribution and its MIT license are included under `assets/material-icon-theme/` and described in `README.md`.
- **File → Open Folder…** replaces the project root; **Project → Add Folder to Project…** imports another root. The primary root's `.sublemon.json` stores workspace roots and the manually launched LSP command.
- The stdio LSP client supports initialization, shutdown, server-request replies, workspace-folder updates, open/change/close synchronization, and published diagnostics.
- Navigation is implemented for definition (F12), declaration, type definition, implementation, references (Shift+F12), and back (Alt+Left). It handles both `Location` and `LocationLink`, and translates Unicode character offsets to LSP UTF-16 positions.
- Ctrl+Space requests completion; results support keyboard/mouse selection and text edits. **Goto → Show Hover** displays hover text. Reference results appear in a selectable list.
- **File → Import Sublime Package…** safely extracts `.sublime-package` ZIP archives into `.sublemon/packages/<name>` and adds the extracted directory to the project tree. Package contents are data only; package code is not run. The imported directory is Git-ignored. Extraction limits and dependency attribution are documented in `README.md`.
- See `README.md` for controls and `.sublemon.json` behavior. Remaining LSP work includes snippets, rename, code actions, semantic tokens, and richer diagnostic presentation.

## Verification and next steps

- `cargo fmt`, `cargo check --offline`, and `git diff --check` passed for this change.
- The GUI was not launched for manual scrolling verification. Confirm wheel/trackpad scrolling through a long file and ensure the gutter remains aligned, in both word-wrap modes; also check cursor auto-scroll and long-line behavior.
- Package import has only been compile-checked. Manually import a valid archive and malformed archives (traversal entries, symlinks, oversized content), and confirm extracted files appear in the tree while `.sublemon/packages/` remains ignored by Git.
- Exercise representative syntax highlighting across bundled languages and verify the fallback behavior for definitions Syntect cannot parse. Keep `THIRD_PARTY_NOTICES.md` and source package notices synchronized when updating the upstream snapshot.
- Project/session restoration beyond roots and the LSP command remains future work. Keep `PLAN.md`, this file, and `SESSION_LOG.md` current after each work step.

## Clean-room and licensing

Sublemon source is independently authored. Do not use Sublime Text binaries, extracted resources, bundled assets, or disassembly. Material Icon Theme is separately attributed and distributed with its MIT license.
