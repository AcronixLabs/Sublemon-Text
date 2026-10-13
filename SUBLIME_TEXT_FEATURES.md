# Sublime Text: complete feature and workflow reference

This guide is an in-depth reference to Sublime Text's interface, editing model, navigation, file and project handling, search, builds, Git integration, configuration formats, extension API, packaging, and history. It uses the [Wikipedia article](https://en.wikipedia.org/wiki/Sublime_Text) as a starting point and relies on [Sublime Text's official documentation](https://www.sublimetext.com/docs/) for current behavior. Sublime Text is closed-source; this describes observable product capabilities rather than implementation source. “Complete” here means broad coverage of documented features and major workflows, not a literal enumeration of every command, setting, API symbol, or third-party package. Sublime Text changes over time, and details can vary by build, operating system, theme, and installed packages.

## What Sublime Text is

Sublime Text is a cross-platform text and source-code editor for Windows, macOS, and Linux. Its core is a fast, keyboard-oriented editor that can be extended with packages and plugins. It supports many languages and text formats through syntax definitions, and can be used for programming, markup, configuration, prose, and other plain-text work. It is proprietary software; the downloadable evaluation version is usable without a time limit, subject to its license terms. [Wikipedia](https://en.wikipedia.org/wiki/Sublime_Text) · [Official downloads](https://www.sublimetext.com/download)

## Interface

- **Menu bar:** Organizes commands under File, Edit, Selection, Find, View, Goto, Tools, Project, Preferences, and Help. The command system behind these commands is also accessible to key bindings, macros, and plugins.
- **Tabs:** Open files appear as tabs. Tabs can be rearranged, grouped, pinned by workflows or packages, and in Sublime Text 4 multiple tabs can be selected together to display them side by side.
- **Editor view:** The main text area supports one or more editor groups. Groups can be arranged in rows, columns, or grids, so several files can be visible at once.
- **Sidebar:** Shows project folders and files, and can show open files. It supports file and folder operations through its context menus; package themes and plugins can change or extend its presentation.
- **Gutter:** Displays line numbers and visual markers such as changed lines, bookmarks, and diagnostics supplied by packages.
- **Minimap:** A compact overview of the file that indicates the visible region and allows quick scrolling through long documents. It can be hidden.
- **Status bar:** Shows contextual information such as syntax, line and column, indentation, encoding, line endings, and Git status where applicable. Some fields can be clicked to change their setting.
- **Quick panels and overlays:** Searchable lists used by Goto Anything, the Command Palette, symbol navigation, and other commands.
- **Output and find panels:** Docked panels at the bottom of the window display build output, search results, and other command output.
- **Themes and color schemes:** Themes style the application chrome and controls; color schemes style text scopes in the editor. Built-in and third-party choices are customizable.

The interface can be pared down or expanded. **Distraction Free Mode** fills the screen with the document and hides interface chrome such as the sidebar, minimap, and status bar; individual elements can be brought back. [Themes and UI elements](https://www.sublimetext.com/docs/themes.html) · [Distraction Free Mode](https://www.sublimetext.com/docs/distraction_free.html)

## Editing

- **Syntax highlighting and folding:** Language-aware syntax definitions assign scopes to text for highlighting, folding, and other context-sensitive behavior. Sublime Text supports its newer `.sublime-syntax` format as well as TextMate `.tmLanguage` grammars.
- **Multiple selections and cursors:** Edit several places at once. Add the next matching occurrence, skip a match, select every match, split a selected block into one selection per line, or use column selection for rectangular edits. Cursors move and edit independently.
- **Incremental selection:** Expand selections by word, line, scope, or bracket context; shrink or adjust selections with keyboard commands.
- **Find and replace:** Search within a file or across folders, with regular-expression support, case and whole-word options, and replacement. Search can be limited to selected text or filtered by file patterns and folders.
- **Goto Anything:** Fuzzy-search for files, symbols, and line numbers from a compact input. Search terms can be combined with file or line hints to jump quickly to a location.
- **Symbol navigation and indexing:** Project indexing records symbols and references to power Goto Definition, Goto Symbol in Project, and context-aware completion. Results depend on the syntax definition and the files included in the project.
- **Completions:** Suggestions can come from the current file, project indexing, completion files, snippets, and plugins. Project-aware completions learn patterns from the project; suggestions can include labels and explanatory details.
- **Snippets:** Insert reusable templates with tab triggers, placeholders, tab stops, variables, and transformations. Snippets can be restricted to a language scope.
- **Auto-pairing and indentation:** Language-aware editing assists with brackets, quotes, indentation, and related routine typing. Behavior is configurable through settings and packages.
- **Macros and repeatable commands:** Record and replay sequences of editor actions; repeat the last action and bind commands to custom shortcuts.
- **Bookmarks and navigation:** Mark lines and move between them, jump to matching brackets, and navigate between search results, symbols, and build errors.
- **Spell checking:** Built-in spell checking can be enabled for prose and configured by language or scope.
- **Encoding and line endings:** Inspect and change a file's character encoding and newline convention from the menus or status bar.
- **Vintage Mode:** Provides a vi-style modal editing layer, with configurable behavior and key mappings.

See [multiple selection](https://www.sublimetext.com/docs/multiple_selection_with_the_keyboard.html), [completions](https://www.sublimetext.com/docs/completions.html), and [indexing](https://www.sublimetext.com/docs/indexing.html) in the official manual.

## Files, projects, and workspaces

- **Folders and projects:** Open folders directly or save a `.sublime-project` file. Project files are JSON and can define included folders, exclusions, project settings, and project-specific build systems.
- **Workspaces:** A `.sublime-workspace` stores session-specific state such as open files and layout. Multiple workspaces can be used with one project, allowing separate working sessions.
- **Project search and navigation:** Goto Anything, Find in Files, symbol indexing, and completion can operate across the folders included in a window or project. Include and exclude patterns control what is searched and indexed.
- **Session restoration:** Sublime Text can restore open files and unsaved buffers after closing or restarting, according to its hot-exit and session settings.
- **Multiple windows and layouts:** Work with separate windows, or divide one window into editor groups. Move files between groups and use tab multi-select to view a set of files together.

The project definition is generally suitable for version control; workspace state is usually personal and is normally excluded. [Projects documentation](https://www.sublimetext.com/docs/projects.html) · [Tab multi-select](https://www.sublimetext.com/docs/tab_multi-select.html)

## Building, diffs, and Git

- **Build systems:** Run external commands for compilation, testing, linting, transpilation, or execution. `.sublime-build` files configure commands, file and syntax selectors, environment, variants, working directory, and error-matching patterns. Output appears in a panel; recognized errors can be navigated in the source.
- **Incremental diff:** Tracks additions, edits, and deletions against the saved file, with gutter markers, inline changes, hunk navigation, and the ability to revert a changed region. In Git repositories, the comparison target can be the index or `HEAD`.
- **Git status:** The sidebar and status bar can show Git file states and branch information. Context-menu and Command Palette actions can open the repository, history, or blame view in Sublime Merge when installed.
- **Sublime Merge relationship:** Sublime Merge is a separate Git client from the same developer. Sublime Text provides selected integration points, not the full feature set of a dedicated Git client.

[Build systems](https://www.sublimetext.com/docs/build_systems.html) · [Incremental diff](https://www.sublimetext.com/docs/incremental_diff.html) · [Git integration](https://www.sublimetext.com/docs/git_integration.html)

## Customization and extensibility

- **Settings:** Text-based settings control editor behavior, appearance, UI, and application behavior. Defaults can be overridden by user, platform, syntax, project, and view-specific settings, with defined precedence.
- **Key bindings:** Bind keys to commands and parameters, with contexts/selectors to make shortcuts apply only in particular situations. Platform-specific bindings are supported.
- **Menus, themes, and color schemes:** UI menus, controls, and syntax colors can be adapted or replaced using package files and theme definitions.
- **Syntax definitions:** Define file extensions, contexts, matching rules, and scopes. Scopes let color schemes and plugins react to language constructs.
- **Build systems, snippets, and completions:** These are user-editable package resources, so workflows can be tailored without modifying the application binary.
- **Packages and plugins:** Packages add commands, language support, themes, snippets, completions, build systems, and integrations. Plugins are written in Python using Sublime Text's API and can react to events, manipulate views and windows, add commands, and provide UI elements.
- **Package Control:** A widely used community package manager for finding, installing, updating, and removing packages. It is a separate open-source project, not the editor's built-in source code.
- **Safe Mode:** Launch with a clean, sandboxed environment for diagnosing problems caused by user settings or packages.

[Settings](https://www.sublimetext.com/docs/settings.html) · [Package development and API](https://www.sublimetext.com/docs/index.html#package-development) · [Package Control](https://packagecontrol.io/)

## Command Palette and command-line use

The **Command Palette** searches available editor commands by name, making infrequently used actions discoverable without remembering menu paths. Packages can contribute commands to it. **Goto Anything** is the corresponding fast navigation tool for files, locations, and symbols.

The `subl` command-line interface can open files, directories, and projects; target a line and column; create or reuse windows; run commands; open stdin; wait for a file to close; or start in Safe Mode. It can also serve as the `EDITOR` for command-line tools. [Command Line Interface](https://www.sublimetext.com/docs/command_line.html)

## Platform and performance notes

Sublime Text is available for Windows, macOS, and Linux. Sublime Text 4 introduced GPU rendering support, tab multi-select, dark mode support, improved project-aware completion, and updated plugin APIs and runtimes. Hardware acceleration behavior depends on operating system and configuration; software rendering is also available. Check the current [compatibility documentation](https://www.sublimetext.com/docs/os_compatibility.html) for supported systems.

## Commonly mentioned community packages

Packages are optional and may change independently of Sublime Text. Examples often used to extend the editor include:

- **LSP:** Language Server Protocol support for language-server-powered navigation, diagnostics, and completion.
- **SublimeLinter:** A framework for connecting external linters.
- **BracketHighlighter:** Additional matching-bracket and tag highlighting.
- **Sidebar enhancements:** Extra file and folder operations in the sidebar.

These examples are not part of a guaranteed default installation; package quality and compatibility vary. Sublime's built-in indexing and editing features work without them.

## Product and licensing context

Sublime Text is proprietary, while many packages and community tools around it are open source. The application can be evaluated without a time limit, but continued use is governed by Sublime HQ's current license terms. Licensing and supported platforms can change; consult the [official licensing information](https://www.sublimetext.com/buy) and [download page](https://www.sublimetext.com/download) for current details.

## Sources

- [Wikipedia: Sublime Text](https://en.wikipedia.org/wiki/Sublime_Text) — overview, historical versions, and broad feature list.
- [Official Sublime Text documentation](https://www.sublimetext.com/docs/) — current product behavior and configuration reference.
- [Official documentation index](https://www.sublimetext.com/docs/index.html) — usage, customization, platform, and package-development topics.
- [Sublime Text 4 announcement](https://www.sublimetext.com/blog/articles/sublime-text-4) — version 4 feature context.

---

## Deep dive: how the editor is organized

### Buffer, view, sheet, group, window

Sublime's interface is easier to understand when these concepts are kept distinct:

- A **buffer** is the text content being edited. It may have a file on disk, or it may be an unsaved scratch buffer.
- A **view** is an editor presentation of a buffer. A file can be shown in more than one view, and views can have separate view-level settings and selections while referring to the same underlying content.
- A **sheet** is Sublime's UI unit for a tab. Tabs can be transient previews or persistent tabs; opening a previewed file usually reuses the preview tab, while editing it or explicitly pinning/opening it makes it persistent.
- A **group** is one editor region in a window. Groups are arranged by the selected layout. A group may contain many tabs, and a view can be moved or cloned into another group.
- A **window** has its own folder set, project context, groups, panels, and session. Multiple windows can work on different projects or different parts of one project.

This separation explains why closing a tab, closing a view, closing a window, and closing a project are distinct actions. It also explains why user settings, project settings, and view-specific settings have different scopes. The API exposes corresponding `Window`, `View`, and `Settings` objects. [API reference](https://www.sublimetext.com/docs/api_reference.html)

### Project and workspace lifecycle

An open folder can serve as an informal project without a saved project file. Saving a project creates a `.sublime-project` definition and associated `.sublime-workspace` state. The project is the shareable definition; the workspace holds personal session state such as open files, layout, and modifications to project data.

A project file has three top-level areas:

- `folders`: one or more file-system roots, with optional include/exclude patterns, symlink handling, and folder-specific settings.
- `settings`: editor settings that apply in the context of that project. They override ordinary user settings where project settings are allowed, but do not override syntax-specific settings.
- `build_systems`: inline build-system definitions that are available when the project is open.

Projects support multiple roots, making a single window useful for a source tree plus sibling libraries or test data. The folder tree and project search are based on the configured roots. A workspace can be changed without changing the shareable project definition. Recent projects can be switched from the UI; command-line flags can open a specified project. [Projects](https://www.sublimetext.com/docs/projects.html) · [CLI](https://www.sublimetext.com/docs/command_line.html)

### Session saving and scratch files

Sublime Text's hot-exit behavior can preserve unsaved buffers and window state across restarts. This is a recovery and workflow feature, not a replacement for saving important files to a durable location. Settings control whether unsaved data is retained, what happens when the final window closes, and how deleted files are treated during restoration. Scratch buffers permit temporary notes without immediately choosing a filename. A scratch buffer can later be saved normally.

## Deep dive: navigation and discovery

### Goto Anything and its query forms

Goto Anything is a fuzzy-search panel for finding files and locations. On common default keymaps it is opened with `Ctrl+P` on Windows/Linux or `⌘P` on macOS. Its compact query syntax combines a file name with a location hint:

| Query shape | Purpose |
| --- | --- |
| `filename` | Fuzzy match a file in the open folders/project. |
| `filename:120` | Open a file at line 120. |
| `filename:120:8` | Open a file at line 120, column 8. |
| `@symbol` | Search symbols in the current file. |
| `filename@symbol` | Search symbols in a named file. |
| `#word` | Find a word in the current file. |

The exact key bindings and query affordances can be customized. Goto Symbol in Project searches the index for definitions across project folders. The indexer records symbol locations and references as provided by syntax definitions; it does not parse every language with a full compiler-grade semantic model. For richer semantics, language-server packages can supply navigation and completion.

### Command Palette versus Goto Anything

These panels look similar but solve different problems:

- **Command Palette** searches actions: build, change syntax, toggle a panel, install a command from a package, or run a named editor command. The default shortcut is commonly `Ctrl+Shift+P` or `⌘⇧P`.
- **Goto Anything** searches content locations: files, symbols, lines, and words.
- **Quick panels** are the general searchable-list UI component used by these and by plugins. Plugins can supply rows, details, and callbacks.

Commands contributed by packages appear in the Command Palette according to their captions and enablement rules. This makes the palette both a launcher and a discoverability mechanism.

### Find panels, regex, and result navigation

The Find panel searches the active buffer; Find in Files searches one or more directories, project folders, or open files. Find options include case sensitivity, whole-word matching, regular expressions, and wrapping. Find in Files adds a **Where** field for folders and file patterns, and can replace across the selected search scope. Results are shown in an output view with file and line locations that can be opened directly. Search result navigation can move through matches and files.

Regex behavior is based on Oniguruma-compatible syntax in Sublime's search and syntax facilities. Use regex carefully for multi-file replacement: preview matches first, constrain the Where scope, and remember that replacement backreferences and escaping have their own syntax. File-pattern matching is a separate glob-like language and is not identical to shell globbing: `*` and `?` are supported, while Bash character classes and `**` globstar are not. A slash changes a pattern from filename matching to path matching. [File patterns](https://www.sublimetext.com/docs/file_patterns.html)

Search scope can be shaped through `file_include_patterns`, `file_exclude_patterns`, folder include/exclude patterns, and binary-file patterns. Project indexing has its own include/exclude patterns, so “searchable” and “indexed for symbols/completions” are related but separately configurable concepts.

### Editing navigation: selection, movement, and transformation

Sublime's editing commands operate on selections. A selection may be empty (a caret) or cover text; there can be many selection regions at once. Most text-editing operations apply independently to every region. Selection commands can expand to words, lines, brackets, scopes, or paragraphs, split regions into per-line cursors, add matching occurrences, or reduce several regions to one.

The common default shortcut family includes:

| Action | Windows/Linux default | macOS default |
| --- | --- | --- |
| Add next occurrence | `Ctrl+D` | `⌘D` |
| Skip next occurrence | `Ctrl+K`, `Ctrl+D` | `⌘K`, `⌘D` |
| Select all occurrences | `Alt+F3` | `⌃⌘G` |
| Split selection into lines | `Ctrl+Shift+L` | `⇧⌘L` |
| Undo selection change | `Ctrl+U` | `⌘U` |

Other built-in text commands cover line movement and duplication, indent/unindent, joining and splitting lines, trimming whitespace, changing case, commenting, transposition, reformatting paragraphs, paste-and-indent, and conversion between indentation styles. Commands can be run from menus, key bindings, the Command Palette, macros, or plugins. Default shortcuts vary by platform and user keymap; the menus and Preferences → Key Bindings show the active mapping. [Multiple selection](https://www.sublimetext.com/docs/multiple_selection_with_the_keyboard.html)

### Column selection

Column selection is implemented as multiple selections rather than a separate editing mode. It can create a rectangular block across uneven lines; operations apply at each row, and the selected row segments can be added to or subtracted from. Mouse gestures differ across operating systems, and keyboard gestures can extend the selection vertically. This is useful for editing fixed-width data, adding prefixes to many lines, and aligning repeated text. [Column Selection](https://www.sublimetext.com/docs/column_selection.html)

## Deep dive: the visible workspace

### Layouts and tab multi-select

The View → Layout controls define editor groups as cells in a row/column grid. A file can be moved into another group, and group navigation is available through commands and shortcuts. Sublime Text 4's tab multi-select gives another way to compare files: selecting several tabs displays them side by side and keeps the set as a selection. It works with tabs, the sidebar, Goto Anything, symbols, and definition navigation. Modifier keys influence whether a navigation target replaces the focused tab, appends to the selected set, or opens beside it. This is distinct from permanently changing the View → Layout grid. [Tab multi-select](https://www.sublimetext.com/docs/tab_multi-select.html)

### Panels and modes

The sidebar, minimap, status bar, tabs, menu, and panels can be shown or hidden individually. Full Screen Mode expands the window; Distraction Free Mode also hides most interface chrome and centers the text. Panels such as Find, Find in Files, Build Output, and Console can be toggled and repositioned by commands or key bindings. The Console is a Python console for inspecting application state and running expressions; it is also where plugin diagnostics and log output are commonly examined.

### Theme, color scheme, and syntax are different layers

Three systems cooperate to render code:

1. A **syntax definition** classifies spans of text and assigns scope names.
2. A **color scheme** maps those scopes to foregrounds, backgrounds, and font styles.
3. A **theme** styles application controls such as tabs, buttons, quick panels, scrollbars, and the sidebar.

Changing a color scheme does not necessarily change the UI chrome; changing a theme does not define the language grammar. Themes and color schemes can use platform or light/dark variants, and Sublime Text 4 added automatic dark/light switching support in its built-in themes and schemes. [Themes](https://www.sublimetext.com/docs/themes.html) · [Color schemes](https://www.sublimetext.com/docs/color_schemes.html)

### Rendering and display

Sublime Text has software rendering and an OpenGL hardware-accelerated renderer. The `hardware_acceleration` setting selects the mode; platform defaults differ. GPU rendering can improve high-DPI and high-resolution rendering, but driver behavior varies. The UI also supports high-DPI theme assets and font settings such as ligatures, font face, size, and line height. [GPU rendering](https://www.sublimetext.com/docs/gpu_rendering.html) · [Font settings](https://www.sublimetext.com/docs/font_settings.html)

## Deep dive: language awareness and completion

### Syntax scopes

A syntax definition assigns scope names such as `source.python`, `keyword.control.python`, or `string.quoted`. Scope names form a hierarchy; selectors can match a scope or a nested portion of it. Scopes are used by color schemes and also by key-binding contexts, snippets, completions, indentation rules, and plugins. The current scope at the caret can be inspected with Show Scope Name. This mechanism allows behavior to be syntax-aware without hard-coding every feature for every language.

The modern `.sublime-syntax` format is YAML-like. A definition declares a name, file extensions, a top-level scope, and named contexts. Rules match regular expressions or include other contexts, assign scopes, and push, set, embed, branch, or pop parsing contexts. Syntax inheritance and embedded syntaxes allow one grammar to reuse or nest another. Legacy TextMate grammars remain supported. These are syntax highlighters, not necessarily full language parsers; they can identify symbols and provide editor behavior but do not by themselves provide type checking. [Syntax definitions](https://www.sublimetext.com/docs/syntax.html) · [Scope naming](https://www.sublimetext.com/docs/scope_naming.html)

### Completion sources and ranking

The completion popup can combine:

- words found in the current buffer;
- project-indexed symbols and context-aware patterns;
- `.sublime-completions` JSON files;
- snippets and their tab triggers;
- results supplied dynamically by plugins.

Completion entries may carry a kind indicator (for example function, type, variable, or snippet), an annotation, and rich details with links. Scope selectors restrict where an entry is offered. Settings control when the popup appears, whether snippets participate while typing, whether project index data is used, and ranking behavior. A plugin can provide completions dynamically or provide a completion file for static mappings. [Completions](https://www.sublimetext.com/docs/completions.html)

### Indexing and its boundaries

Indexing scans included project files in background work. It supports symbol navigation, references, and context-aware completion. It can be limited with file patterns or disabled for some files. Indexing quality depends on the syntax package's symbol rules, file inclusion, and the language. For compiler-aware references, diagnostics, refactoring, and semantic completion, a Language Server Protocol package can connect Sublime Text to an external language server; that behavior belongs to the package and server.

## Deep dive: configuration files and precedence

Sublime Text stores most customization as text files inside Packages/User or project files. Common resource types include:

| Resource | Purpose and format |
| --- | --- |
| `.sublime-settings` | JSON settings for editor, syntax, package, project, or mode behavior. |
| `.sublime-keymap` | JSON arrays of key bindings, commands, arguments, and optional contexts. |
| `.sublime-menu` | JSON menu and context-menu entries. |
| `.sublime-commands` | JSON command-palette entries for commands and arguments. |
| `.sublime-snippet` | XML snippet with trigger, scope, description, and insertion text. |
| `.sublime-completions` | JSON static completion list with a scope selector. |
| `.sublime-syntax` | YAML-style syntax grammar; TextMate grammars are also supported. |
| `.sublime-color-scheme` | JSON scope-to-style rules for text rendering. |
| `.sublime-theme` | JSON rules for UI controls and layout styling. |
| `.sublime-build` | JSON process/build configuration. |
| `.sublime-project` | JSON project folders, settings, and inline build systems. |
| `.sublime-workspace` | Session and workspace state, generally generated/maintained by Sublime. |

The Preferences → Settings view presents defaults beside the user override file. Settings are layered: defaults and platform defaults are combined with user settings, project settings, syntax-specific settings, and buffer/view-specific settings according to setting category and precedence rules. Not every setting is valid at every layer: global UI and application-behavior settings cannot be controlled by a syntax-specific file or a project's editor settings. The left pane in Preferences is descriptive; user changes belong in the right-side override. [Settings](https://www.sublimetext.com/docs/settings.html)

Key bindings are merged from packages' keymaps and user keymaps. A binding can specify a sequence of keys, a command, arguments, and contexts. Contexts can test the current scope, selection, panel, or other state. This permits one key to mean different things in different situations. Platform-specific keymap files are available. For ambiguous shortcuts, inspect the active key bindings rather than assuming another operating system's mapping.

File patterns are reused by search, project roots, indexing, and binary-file exclusion. A pattern without `/` matches a file or folder name; a path pattern with `/` applies to a path, and `//` denotes a path relative to a project root. Since these rules aren't Bash glob syntax, use the official reference when building complex include/exclude patterns. [File patterns](https://www.sublimetext.com/docs/file_patterns.html)

## Deep dive: build systems and external tools

A build system is a declarative adapter from the editor to an external process. It does not generally include a compiler. It can run a program, pass the active filename and project variables, set a working directory and environment, and capture output. The `exec` target covers common process use; custom targets can be implemented as Python window commands.

Build definitions can use syntax selectors, filename patterns, or project key files to decide when they apply. Variants provide related actions such as debug builds, release builds, or test runs. Platform-specific options let one definition choose different commands on Windows, macOS, and Linux. `file_regex` patterns identify filenames and line/column locations in output; clicking or pressing next/previous result jumps to the source. A process can be cancelled, and interactive builds can expose an input panel. The output panel's syntax, encoding, wrapping, and environment are configurable. [Build systems](https://www.sublimetext.com/docs/build_systems.html)

The build panel is also useful as a general task runner for formatters, linters, tests, and scripts, but complex long-lived process management may be better handled by a dedicated plugin or terminal. Sublime Text's core product is an editor with process integration, not a full integrated terminal or debugger.

## Deep dive: Git and change review

Incremental diff computes changes while editing and marks added, changed, and deleted lines in the gutter. It can compare to the saved file, the Git index, or Git `HEAD`, depending on configuration. Inline diffs reveal changed text within a line, navigation steps between hunks, and revert commands restore selected changes. This works even when the file is not in a repository when diffing to disk.

When a file is inside a Git working tree, built-in status indicators can show branch and file-state information, including untracked, modified, staged, deleted, and conflicted states. Sidebar badges can summarize folder contents. The built-in integration links into Sublime Merge for repository view, file and folder history, line history, and blame. Sublime Text intentionally focuses on editor-relevant signals and handoff; commit management and broader repository operations are Sublime Merge's territory. [Git integration](https://www.sublimetext.com/docs/git_integration.html) · [Incremental diff](https://www.sublimetext.com/docs/incremental_diff.html)

## Deep dive: package and plugin ecosystem

### What a package can contain

A package is a collection of resources. It can provide syntaxes, settings, keymaps, menus, snippets, completions, build systems, themes, color schemes, commands, and Python plugins. Sublime's built-in packages ship many language grammars and editing defaults. User packages live separately so upgrades do not overwrite personal changes. Packages can be disabled using settings, which is useful for troubleshooting.

### Package Control and package provenance

Package Control is the common third-party package manager. It can install, update, remove, and disable packages from package repositories and channels. Packages may be distributed through Git hosting, package channels, or custom sources. Package Control itself and many packages are open-source projects; that does not make Sublime Text's core application open source. Package availability, maintenance, compatibility, and trustworthiness depend on each package author and repository.

### Plugin command classes and event listeners

Plugins use Sublime's Python API. Common command classes include:

- `TextCommand`: operates on a view and its text/selection.
- `WindowCommand`: operates on a window, groups, folders, and panels.
- `ApplicationCommand`: operates at application scope.
- Event listeners: observe events such as loading, saving, modifying, or changing a view.

Commands can be invoked from keymaps, menus, `.sublime-commands`, build definitions, and other plugins. A plugin can show input panels and quick panels, create output panels, add regions/annotations/popups, inspect scopes, search text, and modify buffers through edit transactions. The API also exposes windows, views, sheets, selections, settings, syntax, packages, commands, and asynchronous scheduling. API calls that touch UI state must respect the editor's main-thread rules; background work should use the asynchronous mechanisms and return UI updates safely.

Plugins run in a separate `plugin_host` process, separated from the editor UI process so a crashing plugin host need not immediately take down the editor. Plugins in a package share that package's selected Python environment. The official API Environments page documents Python 3.3 compatibility and newer Python runtimes: Python 3.8 was introduced in build 4050, then replaced by Python 3.14 starting in build 4205. The docs state that Python 3.3 is deprecated and scheduled for removal in 2026; exact availability depends on the installed build. Package authors choose a runtime with a `.python-version` file and should consult API Environments and the Plugin Porting Guide. [API reference](https://www.sublimetext.com/docs/api_reference.html) · [API environments](https://www.sublimetext.com/docs/api_environments.html) · [Plugin porting guide](https://www.sublimetext.com/docs/plugin_porting_guide.html)

### UI supplied by plugins

Plugins can contribute items to the Command Palette, menus, quick panels, input panels, completion lists, popups, and phantoms (inline UI attached to text). Minihtml is a constrained HTML/CSS-like rendering system used in some popups and phantoms; it is not a full browser engine. Themes can style plugin minihtml content through color-scheme CSS settings. Plugins may also add gutter regions and annotations, which is how linters and navigation helpers can mark source lines.

## Deep dive: command-line operation

The `subl` CLI is a front end to the running application. It can accept file and directory paths, line/column suffixes, project selection, new-window requests, folder additions, command execution, background opening, and wait semantics. Passing `-` reads standard input; with output redirection supported by the CLI, a transformed buffer can be written back to stdout. `--safe-mode` starts with a clean sandboxed user environment. On Linux tarball installs, the executable can be made available as `subl` by a symlink or wrapper on `PATH`.

Setting `EDITOR='subl -w'` lets command-line programs launch Sublime Text and wait for the file to be closed before continuing. This is useful for Git commit messages and other tools that expect a synchronous editor. Exact flags differ by platform and release; `subl --help` documents the installed build. [CLI documentation](https://www.sublimetext.com/docs/command_line.html)

## Deep dive: safety, troubleshooting, and data handling

- **Safe Mode** (Sublime Text 4+) uses a separate data directory, disables customizations, and erases that Safe Mode data directory at startup. It helps determine whether packages or preferences are causing a problem while leaving the normal data directory intact. Do not store important files or customizations in the Safe Mode directory. It only starts if all normal Sublime windows/the app are fully closed.
- **Revert to a fresh state** and separate data directories are documented for recovering from damaged settings or testing configurations. Back up user packages and settings before replacing or deleting them.
- **Package disablement** can isolate a faulty extension without uninstalling the whole application.
- **Session recovery** can restore unsaved buffers, but users should still save important work to files and use version control or backups for durable recovery.
- **Package code** runs inside the editor's plugin environment and can access files or launch processes according to its API and operating-system permissions. Treat third-party plugins as code from their authors; review sources and provenance when security matters.
- **Project inclusion patterns** influence what is shown, searched, and indexed. Excluding build output and dependency trees can improve relevance and indexing performance.

Official docs: [Safe Mode](https://www.sublimetext.com/docs/safe_mode.html) · [Fresh state](https://www.sublimetext.com/docs/revert.html)

## Built-in command families

The menus expose many commands; the Command Palette provides a searchable route to the same command system. Names and shortcuts may change with keymaps, but common command families include:

- **File:** create, open, close, save, save as, revert, print, open recent, open a folder/project, and manage views. Hot-exit/session settings affect recovery of unsaved buffers.
- **Edit:** undo/redo, cut/copy/paste, paste and indent, line operations, case conversion, whitespace trimming, comment toggles, transpose, paragraph reflow, and snippet insertion.
- **Selection:** select all, expand/shrink selection, add/remove cursors, select matching occurrences, split into lines, and column-selection actions.
- **Find:** find, replace, incremental find, find in files, find next/previous, find all, use selection as search text, and control regex/case/word options.
- **View:** toggle sidebar, minimap, status bar, tabs and panels; change layouts and groups; change zoom and screen modes; control wrapping, line numbers, whitespace, indentation guides, and diff display.
- **Goto:** navigate to files, symbols, definitions, references, lines, words, bookmarks, and build results.
- **Tools:** build, choose a build system, run variants, cancel a build, record/play a macro, and visit the Console.
- **Project:** add/remove folders, save/open/close projects, switch recent projects, and edit project settings.
- **Preferences:** open settings, key bindings, themes, color schemes, syntax resources, and package-related controls.

There is no single exhaustive compiled list of all built-in commands in the manual. The installed `Default` package's menus and keymaps reveal the commands available in a particular build; commands can also be supplied by packages. [Key bindings documentation](https://www.sublimetext.com/docs/key_bindings.html)

## Common editing scenarios

### Change the same token in many places

Select a word and add each next occurrence, skip occurrences that should remain untouched, or select every occurrence. Typing edits all carets at once. For a predictable regular structure, Find/Replace with a carefully scoped regex may be clearer. Undo treats text edits as edits while selection history can be stepped back separately.

### Add or remove text across many lines

Select the line range and split into one selection per line, or make a rectangular column selection. Insert a common prefix/suffix or edit aligned fields simultaneously. This is often faster and less error-prone than a one-off macro for a small repetitive transformation.

### Compare related files

Use tab multi-select to show two or more selected tabs in adjacent panes, or choose a fixed multi-group layout when you want persistent panes. The files remain normal tabs and can be deselected back to the focused tab.

### Find a definition and inspect references

Use Goto Definition on the symbol under the caret, then Goto Symbol in Project or a references command when the syntax index can identify the relevant symbol. If results are incomplete for a language, configure an LSP package and compatible language server; syntax indexing is not equivalent to a language server.

### Run a project command and jump to an error

Create or select a `.sublime-build` definition, run Build, and ensure the output format is covered by a `file_regex`. Select an error result or navigate next/previous result to jump to the source location. The command itself runs in the host operating system; install the compiler/test runner separately.

### Review only the current file's changes

Enable incremental diff markers, inspect inline hunks, step between changes, and revert a hunk if needed. In a repository, set the target to the index or `HEAD` depending on whether the review should include staged content or compare against the commit.

## Version history and feature evolution

The Wikipedia article gives this high-level history; the dates below describe major milestones, while maintenance builds continued afterward:

- **Sublime Text 1:** Began as a Windows editor; early versions established tabs, syntax highlighting, and extensibility. Version 1.3 added Find in Files, richer find/replace, Goto Symbol, and expanded APIs.
- **Sublime Text 2:** Added macOS and Linux support, Retina display support, minimap, multi-selection and column editing workflows, snippets, build integration, and broader customization. This era established many of the interface patterns users associate with Sublime.
- **Sublime Text 3:** Added project symbol indexing, Goto Definition, Goto Symbol in Project, faster package loading, and workspace/project improvements. Version 3.2 added Git status integration and incremental diff.
- **Sublime Text 4:** Released in 2021. Major additions included tab multi-select and split views, refreshed Default/Adaptive themes with dark-mode switching, project-aware context-sensitive completion, improved syntax engine, GPU rendering, Apple Silicon and Linux ARM64 support, Safe Mode, and an expanded Python API. It also improved TypeScript/JSX/TSX support and introduced compatibility handling for older plugin API environments.

This is a feature-oriented summary, not a complete changelog. Sublime's official release notes and changelog are the authoritative source for build-specific changes. [Wikipedia history](https://en.wikipedia.org/wiki/Sublime_Text) · [Sublime Text 4 announcement and full changelog](https://www.sublimetext.com/blog/articles/sublime-text-4) · [Sublime Text 3 announcement](https://www.sublimetext.com/blog/articles/sublime-text-3-beta)

## Practical capability boundaries

Sublime Text is highly extensible, but the distinction between built-in and extension-provided behavior matters:

- Built-in syntax highlighting and symbol indexing do not guarantee full semantic analysis for every language.
- Compiler diagnostics, refactoring, code navigation beyond indexed syntax symbols, and language-specific completions often come from LSP packages and external servers.
- Build systems launch external tools; Sublime does not bundle every compiler, test runner, formatter, or debugger.
- The Git UI displays selected status and diff information and links to Sublime Merge; it is not the entire feature set of a Git GUI.
- Package Control is community-maintained and separate from the core application.
- Themes and plugins can substantially change the interface and available commands, so screenshots and shortcuts from other installations may not match.
- Sublime Text is proprietary and its binaries do not provide the original editor source code. Its package formats and Python API allow substantial customization without access to the application implementation.

## Expanded reference links

- [Official documentation home](https://www.sublimetext.com/docs/)
- [Usage topics and complete documentation index](https://www.sublimetext.com/docs/index.html)
- [Settings and precedence](https://www.sublimetext.com/docs/settings.html)
- [Key bindings](https://www.sublimetext.com/docs/key_bindings.html)
- [Projects and workspaces](https://www.sublimetext.com/docs/projects.html)
- [Indexing and symbol navigation](https://www.sublimetext.com/docs/indexing.html)
- [Completions](https://www.sublimetext.com/docs/completions.html)
- [Multiple selections](https://www.sublimetext.com/docs/multiple_selection_with_the_keyboard.html)
- [Column selection](https://www.sublimetext.com/docs/column_selection.html)
- [Build systems](https://www.sublimetext.com/docs/build_systems.html)
- [Git integration](https://www.sublimetext.com/docs/git_integration.html)
- [Incremental diff](https://www.sublimetext.com/docs/incremental_diff.html)
- [Themes](https://www.sublimetext.com/docs/themes.html)
- [Color schemes](https://www.sublimetext.com/docs/color_schemes.html)
- [Syntax definitions](https://www.sublimetext.com/docs/syntax.html)
- [Plugin API reference](https://www.sublimetext.com/docs/api_reference.html)
- [API environments](https://www.sublimetext.com/docs/api_environments.html)
- [Plugin porting guide](https://www.sublimetext.com/docs/porting_guide.html)
- [Operating-system compatibility](https://www.sublimetext.com/docs/os_compatibility.html)
- [Wikipedia: Sublime Text](https://en.wikipedia.org/wiki/Sublime_Text)
