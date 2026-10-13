## SubLemon Text

Text editing... done correctly.


Just face it. Most text editors are lemons. VS Code, Cursor, VS, Zed, Lapce, Neovim (btw), EMACS (Eventually Must Aquire all Computer System-resources), IntelliJ, Notepad++, and even tExT eDiToR. 

Only one editor is everything I need:
**Sublime Text** 




## Backstory 
I installed Sublime a couple weeks ago to try the evaluation. And... it had all the feautures: Autocomplete, Rust support, Typescript, C, and Python support. But some things were missing.
Namely:
`Zig` support. 
`Assembly` support.
So we at Acronix created SubLemon text!

## Third-party assets

The project tree uses icons from [Material Icon Theme](https://github.com/material-extensions/vscode-material-icon-theme), version 5.39.0, distributed under the MIT License. The original SVGs, rasterized PNGs, and license are in `assets/material-icon-theme/`; see its `ATTRIBUTION.md` for details.

## Project folders and language servers

- Use **File → Open Folder…** to replace the current project root, or **Project → Add Folder to Project…** to import another folder. Enter its full path in the dialog. Roots and the server command are stored in `.sublemon.json` in the primary project folder.
- Use **Project → Start Language Server…** to launch an LSP server command (for example, `rust-analyzer`). Sublemon initializes the server, synchronizes open files, and shows published diagnostics from the status bar tooltip. The command is started only after you choose this menu action. Stop it from the Project menu.
- Use **Goto → Go to Definition** or F12; the same menu offers declaration, type definition, implementation, references, and back navigation. Use Shift+F12 for references, Alt+Left to go back, Ctrl+Space for completions, and **Goto → Show Hover** for hover details.

LSP navigation requests are translated from editor offsets to UTF-16 protocol positions. Completion results can be selected by mouse or keyboard, and server-provided edits are applied to the buffer. The integration also supports hover contents, references lists, diagnostics, workspace-folder updates, and persistent per-project roots/server command. Full snippet expansion, code actions, rename, and richer diagnostic rendering remain unfinished.

## Editing shortcuts

- `()`, `[]`, `{}`, quotes, and backticks are automatically paired. Typing a matching closer when it is already under the caret moves past it; selected text is wrapped by the pair.
- Press `Ctrl+Shift+V` to toggle the built-in Markdown preview. It renders common heading levels, list items, block quotes, and fenced-code markers in a scrollable view; press the shortcut again to return to editing.

## Rendering and large-file overhead

The native app selects eframe's `wgpu` renderer, which uses a supported graphics adapter for UI rendering. The editor also avoids making a full copy of the active document just to measure and draw the line-number gutter. Actual GPU use depends on the host's graphics stack; rendering performance has not been benchmarked across machines.

Use **View → Light Theme** to switch between light and dark editor palettes. **File → Open → File…** and **File → Open → Folder…** open native system pickers. Folder selection replaces the current project root; use **Project → Add Folder to Project…** to add another root.

## `.sublime-package` archives

Choose **File → Import Sublime Package…** and enter the archive path. Sublemon safely extracts the ZIP archive under `.sublemon/packages/<package-name>` in the primary project and adds that directory to the project tree. Imported files are ignored by Git so third-party package contents are not accidentally committed. Extraction rejects unsafe paths and symbolic links, and limits archives to 10,000 entries, 64 MiB per file, and 256 MiB total unpacked data. Package contents are available as files for browsing; Sublime Text plugins and other package code are not executed by Sublemon.

ZIP archive support uses the Rust `zip` crate under its MIT license. Its dependency and license metadata are recorded in `Cargo.toml` and `Cargo.lock`.

## Shipped language packages

Sublemon includes the 48 package folders from [sublimehq/Packages](https://github.com/sublimehq/Packages), snapshot `f3ab97b3a1440ecf2645dd287f37c7e9807c1d45`. That repository contains syntax-highlighting packages; it does not contain Sublime's LSP client. All package data and upstream license notices are retained under `assets/sublime-syntax-packages/`. The repository license allows redistribution except where an individual file has its own notice; those notices are kept with their files.

Sublemon compiles compatible `.sublime-syntax` definitions into a bundled Syntect syntax set and uses an independently authored Sublemon color theme. At this revision, Syntect accepts 78 of 202 syntax files directly; the other definitions remain bundled but use a fallback grammar when available or plain text. Rebuild the syntax database after changing the bundled files with `cargo run --bin rebuild_syntax_data`. Sublemon does not execute package plugins. The separate [Sublime LSP project](https://github.com/sublimelsp/LSP) is not bundled; Sublemon's LSP client remains its own implementation.

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for source revisions, licenses, and asset attribution.
