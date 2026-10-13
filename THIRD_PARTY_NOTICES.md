# Third-party notices

## SublimeHQ syntax package data

`assets/sublime-syntax-packages/` contains the package folders from [sublimehq/Packages](https://github.com/sublimehq/Packages), commit `f3ab97b3a1440ecf2645dd287f37c7e9807c1d45` (2026-10-06). The upstream repository's `LICENSE` grants broad permission to copy, use, modify, sell, and distribute, except files with their own license notices. Those notices remain alongside the files; in particular, the Rust package's `LICENSE.txt` is preserved. The upstream README and revision record are included with the assets.

The app consumes syntax data only. It does not run Sublime package plugins or use Sublime's LSP implementation. The linked repository describes itself as syntax-highlighting packages; the separate [Sublime LSP repository](https://github.com/sublimelsp/LSP) is not included.

## Syntect

Sublemon uses [syntect](https://github.com/trishume/syntect) 5.3.0 to parse syntax definitions and produce token scopes. Syntect is MIT-licensed. Its license is available in the Cargo registry and package metadata.

## Material Icon Theme

The project tree uses Material Icon Theme v5.39.0. Its assets and MIT license are included under `assets/material-icon-theme/`; see `ATTRIBUTION.md`.

## zip

The `.sublime-package` archive importer uses the `zip` crate 9.0.2, which is MIT-licensed. Dependency version and metadata are recorded in `Cargo.toml` and `Cargo.lock`.

## rfd

Native file and folder pickers use `rfd` 0.17.2, licensed under MIT. Dependency metadata is recorded in `Cargo.toml` and `Cargo.lock`.
