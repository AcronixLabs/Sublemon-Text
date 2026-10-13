# Session log

## 2026-10-13 — light theme, native open dialogs, performance plan

- Added a View-menu light theme toggle with coordinated UI/editor and syntax colors. Dark remains the default; saving the selected theme is future work.
- Replaced the path-entry actions for File → Open → File/Folder with native system dialogs through `rfd`; folder open changes the project root while Project → Add Folder remains the multi-root operation.
- Added `rfd` 0.17.2 and MIT attribution to third-party notices.
- Expanded PLAN step 10 with fixed workload sizes, baseline metrics and targets, visible viewport/incremental syntax work, async filesystem tasks, and an engine rewrite gate based on profiling.
- Validation: `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. No GUI/picker check or performance profiling was performed.

## 2026-10-13 — renderer selection and editor allocation reduction

- Configured eframe to use its available `wgpu` renderer for GPU-accelerated UI rendering. Hardware adapter use depends on system support; no runtime GPU measurement was made.
- Removed a complete active-document string clone from per-frame line gutter calculations, retaining only per-line character counts. Preview rows are prepared only in preview mode.
- Documented rendering caveats in README and updated HANDOFF.
- Validation: `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. No benchmark or GUI profiling was run.

## 2026-10-13 — auto-pairs and Markdown preview

- Added automatic pairing for parentheses, brackets, braces, quotes, and backticks in the editor; selections are wrapped and existing closing characters are skipped over.
- Added a `Ctrl+Shift+V` toggle for a scrollable lightweight Markdown preview. It styles headings, list items, block quotes, and fenced-code markers; full CommonMark and inline formatting are not implemented.
- Documented the feature and shortcut in README and updated HANDOFF.
- Validation: `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. GUI interaction was not manually exercised.

## 2026-10-13 — bundled language packages and syntax highlighting

- Fetched the user-requested `sublimehq/Packages` repository at commit `f3ab97b3a1440ecf2645dd287f37c7e9807c1d45` and included all 48 package folders (202 `.sublime-syntax` files) under `assets/sublime-syntax-packages/`. Preserved the repository license and Rust package's separate MIT license; added upstream revision and third-party notices.
- Verified from the repository README that this source is syntax-highlighting packages; Sublime's LSP implementation is a different repository. Sublemon continues to use its own LSP client and does not run Sublime plugins.
- Added a Syntect-backed editor layouter and independently authored Sublemon syntax colors. Built a compact syntax database into `assets/sublemon-syntaxes.packdump`; Syntect parsed 78 syntax files directly, while unsupported definitions remain included and fall back to Syntect defaults or plain text.
- Added `cargo run --bin rebuild_syntax_data` for regenerating the compiled database. Parsing all source grammar files at normal app build time took several minutes, so the compiled packdump is checked in and standard builds no longer compile the grammar corpus.
- Updated `AGENTS.md`, README, plan, handoff, and third-party notices to document the authorized data-vendoring exception and the LSP distinction.
- Validation: `cargo fmt` and `cargo check --offline` passed. No GUI or per-language highlighting review was performed.

## 2026-10-13 — `.sublime-package` archive import

- Added **File → Import Sublime Package…**. It extracts a selected ZIP package to `.sublemon/packages/<package-name>` under the primary project and adds that directory to project roots for browsing.
- Added safeguards for archive entry paths, symbolic links, duplicate/existing destinations, 10,000-entry maximum, 64 MiB per entry, and 256 MiB aggregate unpacked content. Extraction occurs in a temporary staging directory and is renamed into place after success.
- Package contents are treated as data; Sublemon does not execute Sublime Text package code. Added `.sublemon/packages/` to `.gitignore` to avoid accidental redistribution.
- Added the `zip` dependency and documented its MIT/Apache-2.0 license in README.
- Validation: `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. GUI/archive import was not manually exercised.

## 2026-10-13 — editor text scrolling

- Fixed the editor layout by placing the line-number gutter and multiline text editor inside one shared vertical `ScrollArea`.
- The scroll content height is calculated from the document's visual rows, accounting for word wrapping, so the complete document can scroll while gutter rows remain vertically aligned.
- Validation: `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. GUI scrolling was not manually exercised.

## 2026-10-13 — LSP navigation, completion, hover, and project settings

- Added LSP navigation requests for definitions, declarations, type definitions, implementations, and references. Results accept `Location` and `LocationLink`; navigation opens the target file, restores the cursor, and records a back location. Added F12, Shift+F12, and Alt+Left actions.
- Added UTF-16 position conversion between editor character offsets and LSP coordinates for Unicode text.
- Added Ctrl+Space completion requests and a selectable results popup. Completion items can apply server-provided edits or replace the current identifier prefix.
- Added hover requests and a hover-content window reachable through **Goto → Show Hover**.
- Added per-project `.sublemon.json` persistence for workspace roots and the configured language-server command. Opening/replacing a project reads its settings; importing a folder and starting/configuring a server save them.
- Updated README usage, PLAN milestone notes, and this handoff to reflect the implementation and remaining LSP features.
- Validation: `cargo fmt`, `cargo check --offline`, and `git diff --check` passed. No real server or GUI interaction was exercised.

## 2026-10-13 — project scrolling, folders, attribution, and LSP foundation

- Wrapped the project tree in a virtualized vertical scroll area using `show_rows`; added a two-second directory listing cache so expanded folders are not rescanned every frame.
- Implemented **File → Open Folder…** to replace the current root and **Project → Add Folder to Project…** to import multiple directory roots.
- Added a stdio JSON-RPC LSP client with initialization, server-request replies, workspace folder notifications, document synchronization, and published diagnostics.
- Confirmed Material Icon Theme attribution and bundled its MIT license.
- `cargo fmt` and `cargo check --offline` passed. No real language server or GUI interaction was exercised.

## Earlier icon work

- Bundled Material Icon Theme v5.39.0 file/folder assets and generated associations; see `assets/material-icon-theme/ATTRIBUTION.md` and the repository history.
