# Sublemon: clean-room Rust editor plan

## Goal and constraints

Create **Sublemon**, an original Rust desktop text editor with a visually exact recreation of Sublime Text's standard UI, as explicitly requested by the user. The implementation must remain independently authored and source-incompatible: use Sublemon's name and user-provided root `icon.png`, and draw matching interface elements without copying application code or extracting proprietary UI/theme resources.

This is a staged plan. Each milestone should leave a runnable, coherent application. Validate behavior with manual acceptance checks and automated tests where useful before marking a milestone complete. Do not use Sublime Text binaries, extracted resources, or disassembly as implementation input. The public feature notes and official documentation are allowed only as descriptions of behavior to independently implement.

## Architecture direction

- **Language:** Rust, edition 2024.
- **Native UI shell:** eframe/egui, with its wgpu renderer and independently authored custom editor layout. Keep UI, document, and language-service layers modular.
- **Core model:** Keep documents, file identity, encoding, line endings, selections, undo history, and dirty state separate from frame/UI code. Large-file mode currently uses egui `TextEdit`; a virtualized rope-backed buffer is still required to eliminate full-buffer layout cost.
- **Filesystem and settings:** Use explicit, testable modules for file I/O, workspace state, settings, and project configuration. Never silently overwrite or normalize user data.
- **Long-running work:** Run directory scans, indexing, builds, and external-tool communication asynchronously, report progress, and keep editing responsive.
- **Dependencies:** Prefer established, actively maintained Rust crates with compatible licenses. Recheck versions and licenses before release and document them.

## Step-by-step implementation

### 1. Create the native application foundation — in progress

- Create the Rust Cargo application `sublemon` at the repository root.
- Add the native window, app lifecycle, initial layout shell, and a minimal in-memory editable document.
- Embed the supplied `icon.png` as the Sublemon window icon.
- Add clean-room `AGENTS.md` guardrails and this milestone plan.
- Acceptance: the project has a coherent source layout, starts as a native app, displays the Sublemon identity/icon, and accepts text input in a scratch buffer.

### 2. Build the document and file model

- Define document identity, path, buffer contents, encoding, line-ending style, syntax/language identity, dirty state, and read-only state.
- Implement safe open/save/save-as/revert flows, error reporting, external-change detection, and conflict-safe save behavior.
- Support untitled buffers and explicit close prompts for modified documents.
- Add model-level tests for encoding, newlines, path identity, dirty state, and save failures.

### 3. Establish a capable editing engine

- Prototype text storage and rendering against large files and long lines; decide between the UI toolkit's text widget and a purpose-built editor view.
- The active frontend uses egui's `TextEdit` with Sublemon-managed line layout, gutter, scrolling, syntax painting, and document state. Continue profiling large files and preserve cursor, selection, wrapping, and undo behavior during rendering work.
- Implement caret movement, selections, word/line operations, clipboard, undo/redo, indentation, wrapping, and basic keyboard editing.
- Preserve Unicode grapheme boundaries, IME composition, accessibility semantics, and predictable selection behavior.
- Add multi-cursor support only after single-cursor behavior and undo semantics are solid.

### 4. Recreate the editor workspace UI

- Recreate the standard visible arrangement: menu bar, file tabs, file sidebar, editor gutter, code surface, minimap, and status bar.
- Match control spacing, panel proportions, typography, dark theme colors, tab emphasis, and selected/hover states from public visual references.
- Add split editor groups, adjustable layouts, panel visibility, window sizing/restoration, and keyboard navigation.
- Draw all visuals with Sublemon's own code; use `icon.png` for the app window and package it for supported platforms.

### 5. Add navigation and search

- Implement in-file find/replace with literal and regex modes, match highlighting, and next/previous navigation.
- Add project-wide search with include/exclude filters, result preview, and safe replacement.
- Add fuzzy file opening, line/column navigation, symbol navigation, and recent-file/project lists.
- Make searches cancellable and asynchronous for large workspaces.

### 6. Add language awareness

- Keep syntax selection and highlighting in original Sublemon code; do not bundle third-party editor packages or proprietary editor grammar ecosystems.
- Provide bracket matching, code folding, indentation rules, comments, snippets, and completion interfaces.
- Keep syntax highlighting separate from the built-in LSP process and document the current lexical behavior and its limits.
- Design Sublemon-native settings for language behavior without importing editor-specific package formats.

### 7. Implement projects and workspace persistence

- Define a Sublemon project format for multiple roots, excludes, settings, and task definitions.
- Persist window/session state separately from shareable project configuration.
- Add background filesystem watching and project indexing with limits, cancellation, and progress feedback.
- Exclude common generated/dependency folders by sensible defaults while allowing user overrides.

### 8. Add build tasks, diagnostics, and version-control context

- Define configurable external tasks for build, run, test, and lint; parse common diagnostic formats and navigate to source locations.
- Add a process/output panel, environment controls, cancellation, and safe argument passing.
- Integrate Git status and working-tree diffs through a library or Git executable, keeping writes and destructive operations explicit.
- Consider language-server support through LSP only after the document, diagnostics, and asynchronous message models are ready.

### 9. Add extensibility and configuration

- Design a documented settings schema, keybinding system, command registry, and extension API based on Rust-friendly, stable interfaces.
- Decide whether extensions are declarative, subprocess-based, WebAssembly-based, or native plugins; prefer isolation and compatibility over loading arbitrary native code in-process.
- Add import/export and safe-mode behavior for diagnosing user customizations.
- Maintain a dependency/license inventory and extension trust model.

### 10. Accessibility, reliability, and performance — in progress

- Audit keyboard-only workflows, screen-reader/accessibility tree exposure, contrast, scaling, IME, and platform conventions.
- Profile before redesign: build optimized binaries, record cold/warm startup, idle CPU, typing-to-frame p50/p95, scroll frame times, and resident memory on fixed 1 MiB, 10 MiB, and 100 MiB documents. Capture CPU flamegraphs and allocator/RSS snapshots on Linux; repeat on each supported OS before platform claims.
- Set initial targets on the reference machine: idle CPU below 1% of one core after settling; p95 keystroke-to-visible-frame below 16 ms for 10 MiB; 100 MiB open without UI stalls over 100 ms; and avoid retaining multiple full-size copies of a document. Record hardware, OS, build profile, and results so changes can be compared.
- First remove full-buffer work from the frame path: virtualize visible editor and gutter rows, cache line layout/width and syntax results by document revision, and update only affected lines after edits. Use incremental parsing/highlighting and avoid building full-document `LayoutJob`s on every redraw.
- Profile the egui editor and viewport rendering on the same fixed workloads; preserve Unicode, selection, IME, accessibility, and undo behavior while addressing measured bottlenecks.
- Move directory walks, project indexing, and expensive file metadata work off the UI thread with cancellation, bounded work queues, and cache invalidation. Debounce/coalesce LSP change notifications and avoid copying whole buffers for transient diagnostics or UI state.
- Treat GPU rendering as a way to accelerate drawing only. It will not speed up text scanning, syntax parsing, disk I/O, or allocations; measure CPU and GPU paths separately and retain a usable fallback where supported.
- Add crash recovery/autosave policy with clear user expectations after responsiveness and data integrity are measured.
- Exercise Linux, Windows, and macOS builds; account for native file dialogs, clipboard, fonts, and application identity on each platform.

### 11. Packaging and release

- Create platform-native packages and a portable development build.
- Include Sublemon's icon and metadata, licenses/notices for dependencies, and user documentation.
- Verify clean installs, upgrades, workspace migration, and uninstall behavior.
- Publish source and binaries under a license chosen for Sublemon; do not include proprietary Sublime Text material.

## Milestone tracking

| Step | Status | Notes |
| --- | --- | --- |
| 1. Native app foundation | Complete | Cargo project, eframe/egui shell, editable buffer, and supplied icon are implemented; `cargo check --offline` passes. |
| 2. Document/file model | In progress | Added a standalone UTF-8 document model, normalized editing with detected line-ending preservation, dirty tabs, path-based Open/Save/Save As/Revert, read-only protection, modified-file close prompts, and disk-change/overwrite guards. The current path prompt is a basic text-entry dialog; native file pickers, richer encoding support, and dedicated model tests remain. |
| 3. Editing engine | In progress | Active frontend uses egui `TextEdit`, custom gutter rendering, and an independently written lightweight token scanner. Large files load asynchronously, skip syntax coloring and wrapping over 512 KiB, and defer full line-metric refresh after keystrokes. egui still lays out the full buffer; benchmark and consider a virtualized rope editor. |
| 4. Workspace UI | In progress | User requested pixel-level visual fidelity to the supplied target screenshot. The app opens the project README and AGENTS files, shows an expandable project tree beside editor-only tabs, wraps and lightly highlights Markdown, and uses sampled target colors and panel proportions. Removed the nonfunctional text-as-minimap pane after it consumed most of the editor width; the sidebar divider remains draggable, starts at 43% of the viewport, and can collapse to zero width. Moved the gutter into the editor workspace below the tab strip so the tabs align at the sidebar edge as in the reference. Bundled the MIT-licensed Material Icon Theme v5.39.0 pack with filename, extension, and folder associations. The tree uses a virtualized scroll area and short-lived directory cache. Added `scripts/ui_screenshot_test.py` plus `SCREENSHOT_TESTING.md`; the supplied pre-change screenshot scored 41.28% similarity at tolerance 16, below the 80% target. |
| 5. Search/navigation | Not started | |
| 6. Language awareness | In progress | Syntax coloring uses an original lightweight token scanner. A clean-room local LSP process provides lexical completion, hover, local definition/references, document symbols, and delimiter diagnostics for open documents. It does not provide compiler semantics or unopened-file indexing yet. The previously bundled syntax package archive, packdump, and importer have been removed. |
| 7. Projects/persistence | In progress | Open-folder and multi-root import are available. `.sublemon.json` persists project roots. New app state stores recent projects; first run shows Welcome and later runs reopen the latest valid project. |
| 8. Tasks/Git/LSP | In progress | The app starts its own stdio JSON-RPC language service for an open project. It skips files over 512 KiB to avoid competing with large-file editing. Build tasks, Git, semantic analysis, unopened-file indexing, code actions, rename, and richer diagnostics remain. No external language-server or editor-plugin ecosystem is launched. |
| 9. Extensibility/config | Planned | No editor package archive compatibility is included. Design Sublemon-native settings and extension interfaces independently. |
| 10. Reliability/performance | In progress | File reads moved to a worker, duplicate saved text and duplicate open-time hashing were removed, large-file syntax/wrapping and eager gutter painting were reduced. Need repeatable large-file benchmarks and a viewport-based text buffer to address egui's remaining full-document layout cost. |
| 11. Packaging/release | Not started | |

## References

- [egui documentation](https://docs.rs/egui/latest/egui/) and [eframe documentation](https://docs.rs/eframe/latest/eframe/) — active UI toolkit, native window, and rendering APIs.
- [Material Icon Theme](https://github.com/material-extensions/vscode-material-icon-theme) — MIT-licensed file and folder icon assets and association data; license and asset attribution are included in `assets/material-icon-theme/`.
