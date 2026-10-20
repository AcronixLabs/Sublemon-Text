# Sublemon

## UI framework

The active native application uses eframe/egui, with an independently implemented menu bar, project tree and icons, tabs, editor gutter and scrolling, minimap, status bar, theme toggle, file pickers, and Markdown preview.

On first launch, Sublemon shows a welcome screen. Once a project is opened, its path is saved in the app's user-data directory; subsequent launches reopen the most recently used project. The app-data file is newly introduced, so this version's first launch has no previous project and opens the welcome screen.

Sublemon is an original Rust desktop editor with a compact project workspace, tabs, syntax coloring, and a local language service.

## Third-party assets

The project tree uses icons from [Material Icon Theme](https://github.com/material-extensions/vscode-material-icon-theme), version 5.39.0, distributed under the MIT License. The original SVGs, rasterized PNGs, and license are in `assets/material-icon-theme/`; see its `ATTRIBUTION.md` for details.

## Project folders and language servers

Use **File → Open Folder…** to replace the project root, or **Project → Add Folder to Project…** to add another root. Workspace roots are stored in `.sublemon.json`. A small, in-tree Sublemon language service starts automatically while a project is open. It provides local-document completion, hover, definition/references, document symbols, and unmatched-delimiter diagnostics using lexical scans. It does not launch external language servers, load third-party editor packages, or index unopened project files. It is not a compiler or type checker. Release builds can ship the lightweight `sublemon-lsp` companion binary next to the editor.

## Editing shortcuts

- `Ctrl+S` saves the current document, `Ctrl+O` opens a file picker, and `Ctrl+Shift+V` toggles Markdown preview. Brackets and quotes auto-pair, selected text can be wrapped, and typing an existing closer skips over it.

## Rendering and themes

The native shell uses eframe's wgpu renderer. Use **View → Light Theme** to switch palettes. File and folder actions open native pickers. Large documents load on a worker thread, avoid a duplicate saved-text copy, disable syntax coloring and wrapping above 512 KiB, and only paint visible gutter rows. The language service skips synchronization and diagnostics for documents above that size. Large-file performance has not yet been benchmarked across machines; egui's text widget still lays out the full active buffer, so very large files can remain slow while editing or scrolling.

## Syntax highlighting

Syntax styling is implemented in Sublemon with a small token scanner for common programming constructs. It does not load third-party editor package grammars or editor-specific package formats. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for dependency licenses and asset attribution.
