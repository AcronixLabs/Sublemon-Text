# Sublemon project instructions

## Project goal

Build **Sublemon**, an original, clean-room text editor written in Rust. It may implement familiar editor concepts, but its code, interface, branding, and bundled resources must be created independently. The user-provided `icon.png` is approved for Sublemon branding.

## Clean-room rules

- Do not copy, translate, decompile, or adapt Sublime Text application/runtime source or binaries. Do not use the extracted `sublime_text/` installation, the AppImage archive, or generated disassembly as implementation material. Exception authorized by the user on 2026-10-13: vendor the licensed syntax package data from `sublimehq/Packages` under `assets/sublime-syntax-packages/`, retaining upstream license notices and file-specific licenses; use it as grammar data only.
- Do not inspect the installed application's code or bundled resources for implementation details. `SUBLIME_TEXT_FEATURES.md`, official public documentation, and other public descriptions may be used only as functional references; independently design and write all implementation code and UI. The separately maintained Sublime LSP plugin is not the `sublimehq/Packages` repository and is not bundled here.
- The user has explicitly requested a visually exact UI clone. Match the visible layout, control placement, labels, and styling of Sublime Text's standard UI as closely as practical, while retaining the product name **Sublemon** and the user-provided `icon.png` as its app icon. Draw all UI styling and controls independently; do not extract or bundle Sublime Text theme files, logos, icons, or other proprietary assets.
- Prefer original, modular Rust implementations and independently licensed dependencies. Record third-party dependencies and their licenses before distribution.
- Keep Sublemon's source, assets, settings, and build outputs outside the proprietary `sublime_text/` directory. Do not add proprietary archives, extracted files, or disassembly to the Sublemon package.

## Engineering conventions

- Use safe Rust; avoid `unsafe` unless a documented, reviewed platform integration requires it.
- Keep UI state separate from document/model logic as the project grows.
- Preserve text exactly where possible. Make encoding, line-ending, save, and recovery behavior explicit rather than silently transforming user data.
- Treat large files and responsive input as first-class requirements. Avoid blocking the UI thread for disk scans, indexing, builds, or language-server work.
- Keep behavior and architectural decisions documented in `PLAN.md` and update it when milestones change.
- Do not claim a milestone works on a platform until it has actually been built and exercised there.

## Current foundation

- The native desktop UI uses `eframe`/`egui` for the initial Rust shell. This is an architectural starting point, not a promise to force the final text engine into a widget that cannot meet editor requirements.
- `icon.png` at the repository root is the only initial Sublemon brand asset; separately licensed Material Icon Theme and SublimeHQ syntax data are recorded in project notices.
- The clean-room feature notes are informational references, not implementation source.
