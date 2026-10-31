# subcli

`subcli` is a Sublime-like terminal editor built on Neovim. It uses VI modal editing instead of nano-style shortcuts, ships a Sublime-inspired theme and command set, and imports declarative data from Sublime packages you already own.

## Requirements

- Neovim 0.9 or newer
- Optional: `fd` or `rg` for fast file search
- Optional: a clipboard provider for system clipboard integration

Snippet expansion uses Neovim's built-in `vim.snippet` API when available (Neovim 0.10+). On older versions, Tab falls back to normal indentation.

## Run from a checkout

```sh
./subcli/bin/subcli path/to/file
```

The launcher sets `NVIM_APPNAME=subcli` and starts Neovim with `subcli/init.lua`. It does not read your regular Neovim configuration.

## Install

Unix:

```sh
./subcli/install.sh
```

This installs to `$HOME/.local/share/subcli` and `$HOME/.local/bin/subcli` by default. Set `PREFIX` to change the prefix.

Arch:

```sh
cd packaging/subcli/arch
makepkg -si
```

Portable archives are produced by CI as `subcli-unix.tar.gz` and `subcli-windows.zip`. On Windows, run `subcli\bin\subcli.cmd` from the extracted archive.

## Keybindings

| Key | Action |
| --- | --- |
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save As |
| `Ctrl+O` | Open file |
| `Ctrl+N` | New file |
| `Ctrl+W` | Close buffer |
| `Ctrl+Shift+W` | Close window |
| `Ctrl+P` | Open file by name |
| `Ctrl+Shift+P` | Command palette |
| `Ctrl+F` | Find |
| `Ctrl+H` | Replace |
| `Ctrl+G` | Go to line or `file:line:col` |
| `Ctrl+D` | Select next occurrence |
| `Ctrl+/` | Toggle line comment |
| `Ctrl+C` | Copy line or selection |
| `Ctrl+X` | Cut line or selection |
| `Ctrl+V` | Paste |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` | Redo |
| `Ctrl+A` | Select all |
| `Ctrl+Shift+D` | Duplicate line |
| `Ctrl+Shift+K` | Delete line |
| `Alt+Up/Down` | Move line or selection |
| `Alt+H/J/K/L` | Move between windows |
| `Alt+V` | Split vertically |
| `Alt+S` | Split horizontally |
| `F1` | Help |

Window commands that normally live under `Ctrl+W` are available through `Alt+H/J/K/L` and `Ctrl+Shift+W`, because `Ctrl+W` closes the current buffer.

Use `:SubcliHelp`, `:SubcliHealth`, and `:SubcliPackages` for in-editor help and package status. `:SubcliReload` reloads the configuration and package data.

## Sublime package interoperability

`subcli` reads these file types from package roots:

- `.sublime-settings` for supported editor settings
- `.sublime-keymap` for supported keybindings and commands
- `.sublime-snippet` for Tab-triggered snippets
- `.sublime-syntax` for `file_extensions` to Neovim filetype associations

Default package roots:

- Linux: `$XDG_CONFIG_HOME/sublime-text/Packages`, `~/.config/sublime-text/Packages`, `~/.config/sublime-text-4/Packages`, `~/.config/sublime-text-3/Packages`
- macOS: `~/Library/Application Support/Sublime Text/Packages`, `~/Library/Application Support/Sublime Text 3/Packages`
- Windows: `%APPDATA%/Sublime Text/Packages`, `%APPDATA%/Sublime Text 3/Packages`

Override the roots with a platform-separated list:

```sh
SUBCLI_SUBLIME_PACKAGES="$HOME/.config/sublime-text/Packages:$HOME/my-packages" subcli
```

The `User` package is applied after other packages so user settings and keybindings win. Existing Neovim filetype associations are not replaced.

Supported settings include `tab_size`, `translate_tabs_to_spaces`, `word_wrap`, `rulers`, `line_numbers`, `highlight_line`, `draw_white_space`, `trim_trailing_white_space_on_save`, `ensure_newline_at_eof_on_save`, `auto_indent`, `smart_indent`, `detect_indentation`, `tab_completion`, `shift_tab_unindent`, `scroll_past_end`, and `ignored_packages`.

Supported Sublime commands include save/open/close, find/replace, goto line, select next, toggle comment, duplicate/delete/move line, copy/cut/paste, undo/redo, indent/unindent/reindent, join lines, select all, and common overlay commands.

## Limitations

- Sublime Text Python plugins are not loaded or executed.
- `.sublime-commands`, build systems, project files, UI themes, and `.tmTheme` files are not imported.
- Sublime syntax definitions are not executed as grammars. Only `scope` and `file_extensions` metadata are used to associate extensions with Neovim filetypes.
- Unsupported Sublime keybindings are skipped and reported by `:SubcliPackages`.
- Some `Ctrl+Shift` combinations are not distinguishable from their unshifted forms in every terminal. The equivalent `:Subcli...` commands remain available.
- No proprietary Sublime Text assets are bundled or redistributed.

## User configuration

When launched through the `subcli` launcher, an optional `~/.config/subcli/user.lua` is sourced after the built-in configuration. Set `SUBCLI_USER_CONFIG` to use a different file.

Useful environment variables:

- `SUBCLI_SHARE`: path to the directory containing `init.lua`
- `SUBCLI_USER_CONFIG`: path to a user Lua file
- `SUBCLI_SUBLIME_PACKAGES`: package root list
- `NVIM`: Neovim executable to use
- `NVIM_APPNAME`: defaults to `subcli`

## Tests

```sh
./subcli/test.sh
```

The test uses a fixture package and a headless Neovim instance to verify settings, keymaps, snippets, syntax associations, and the default configuration.
