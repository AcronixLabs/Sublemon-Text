# Sublemon project instructions

## Project goal

Build **Sublemon**, an original, clean-room text editor written in Rust. It may implement familiar editor concepts, but its code, interface, branding, and bundled resources must be created independently. The user-provided `icon.png` is approved for Sublemon branding.

## Clean-room rules

- Do not copy, translate, decompile, or adapt Sublime Text application/runtime source or binaries. Do not use the extracted `sublime_text/` installation, AppImage archive, generated disassembly, Sublime package archives, or Sublime package data as implementation material. The previous syntax-data vendoring exception is withdrawn; do not re-add it.
- Do not inspect the installed application's code or bundled resources for implementation details. `SUBLIME_TEXT_FEATURES.md`, official public documentation, and other public descriptions may be used only as functional references; independently design and write all implementation code and UI. For the Rust Sublemon app, do not bundle or execute third-party editor packages, plugins, or language-server ecosystems.
- The user has explicitly requested a visually exact UI clone. Match the visible layout, control placement, labels, and styling of Sublime Text's standard UI as closely as practical, while retaining the product name **Sublemon** and the user-provided `icon.png` as its app icon. Draw all UI styling and controls independently; do not extract or bundle Sublime Text theme files, logos, icons, or other proprietary assets.
- Prefer original, modular Rust implementations and independently licensed dependencies. Record third-party dependencies and their licenses before distribution.
- Keep Sublemon's source, assets, settings, and build outputs outside the proprietary `sublime_text/` directory. Do not add proprietary archives, extracted files, or disassembly to the Sublemon package.

## Subcli (Neovim distribution)

`subcli/` is a separate Neovim-based terminal distribution, approved by the user as a parallel product. The clean-room rules above continue to apply to the Rust Sublemon app and its bundled assets. For `subcli` only:

- Neovim is the editor engine and may be invoked as an external program.
- At runtime, subcli may read user-owned Sublime Text packages (settings, keymaps, snippets, and syntax metadata) for interoperability.
- Do not bundle, redistribute, or commit Sublime Text proprietary assets, package archives, or extracted application data. Import only from paths the user already owns.
- Do not execute Sublime Text Python plugins. Only declarative package data is read.
- The default subcli configuration must remain usable with a stock Neovim and no third-party Neovim plugins.

## Engineering conventions

- Use safe Rust; avoid `unsafe` unless a documented, reviewed platform integration requires it.
- Keep UI state separate from document/model logic as the project grows.
- Preserve text exactly where possible. Make encoding, line-ending, save, and recovery behavior explicit rather than silently transforming user data.
- Treat large files and responsive input as first-class requirements. Avoid blocking the UI thread for disk scans, indexing, builds, or language-server work.
- Keep behavior and architectural decisions documented in `PLAN.md` and update it when milestones change.
- Do not claim a milestone works on a platform until it has actually been built and exercised there.

## Current foundation

- The native desktop UI uses eframe/egui, including the existing custom editor layout, project tree, tabs, status bar, and menus. Floem prototype files may remain in the repository but are not part of the compiled application.
- `icon.png` at the repository root is the only initial Sublemon brand asset; separately licensed Material Icon Theme assets are recorded in project notices. Syntax highlighting is independently implemented.
- The clean-room feature notes are informational references, not implementation source.
- `subcli/` is the terminal Neovim distribution: a Sublime-like modal keymap, theme, command set, and user-owned Sublime package importer. It is intentionally separate from the Rust application.
