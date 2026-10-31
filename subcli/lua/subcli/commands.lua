local M = {}

local function notify(message, level)
  vim.notify("subcli: " .. message, level or vim.log.levels.INFO)
end

local function scratch(name, lines, filetype)
  vim.cmd("enew")
  local buf = vim.api.nvim_get_current_buf()
  vim.api.nvim_buf_set_lines(buf, 0, -1, false, lines)
  vim.bo[buf].buftype = "nofile"
  vim.bo[buf].bufhidden = "wipe"
  vim.bo[buf].swapfile = false
  vim.bo[buf].filetype = filetype or "markdown"
  vim.bo[buf].modifiable = false
  pcall(vim.api.nvim_buf_set_name, buf, name)
  vim.keymap.set("n", "q", "<Cmd>close<CR>", {
    buffer = buf,
    silent = true,
    desc = "Close",
  })
end

local function run_list(command)
  local ok, output = pcall(vim.fn.systemlist, command)
  if not ok or vim.v.shell_error ~= 0 then
    return {}
  end
  return output
end

local function list_files()
  local files = {}
  if vim.fn.executable("fd") == 1 then
    files = run_list({
      "fd",
      "--type",
      "f",
      "--hidden",
      "--exclude",
      ".git",
      "--exclude",
      ".cache",
      "--exclude",
      "node_modules",
      "--exclude",
      "target",
      "--exclude",
      "dist",
    })
  elseif vim.fn.executable("rg") == 1 then
    files = run_list({
      "rg",
      "--files",
      "--hidden",
      "-g",
      "!.git",
      "-g",
      "!.cache",
      "-g",
      "!node_modules",
      "-g",
      "!target",
      "-g",
      "!dist",
    })
  else
    local cwd = vim.fn.getcwd()
    for _, path in ipairs(vim.fn.globpath(cwd, "**/*", false, true)) do
      if vim.fn.isdirectory(path) == 0 and not path:find("/%.git/", 1, true) then
        table.insert(files, path)
      end
    end
  end

  table.sort(files)
  if #files > 20000 then
    local limited = {}
    for index = 1, 20000 do
      limited[index] = files[index]
    end
    files = limited
  end
  return files
end

function M.open_path(path)
  if not path or path == "" then
    return
  end
  vim.cmd("edit " .. vim.fn.fnameescape(vim.fn.expand(path)))
end

function M.write()
  vim.cmd("write")
end

function M.save_as()
  local default = vim.fn.expand("%:p")
  vim.ui.input({
    prompt = "Save as: ",
    default = default,
    completion = "file",
  }, function(input)
    if not input or input == "" then
      return
    end
    local path = vim.fn.fnamemodify(vim.fn.expand(input), ":p")
    local ok, err = pcall(vim.cmd, "saveas " .. vim.fn.fnameescape(path))
    if ok then
      notify("wrote " .. path)
    else
      notify(tostring(err), vim.log.levels.ERROR)
    end
  end)
end

function M.open()
  vim.ui.input({
    prompt = "Open file: ",
    completion = "file",
  }, function(input)
    if input and input ~= "" then
      M.open_path(input)
    end
  end)
end

function M.open_command(opts)
  if opts.args and opts.args ~= "" then
    M.open_path(opts.args)
  else
    M.open()
  end
end

function M.new_file()
  vim.cmd("enew")
end

function M.close_file()
  local ok, err = pcall(vim.cmd, "bdelete")
  if not ok then
    notify(tostring(err), vim.log.levels.WARN)
  end
end

function M.close_all()
  local ok, err = pcall(vim.cmd, "%bdelete")
  if not ok then
    notify(tostring(err), vim.log.levels.WARN)
  end
end

function M.find()
  vim.ui.input({
    prompt = "Find: ",
    default = vim.fn.getreg("/"),
  }, function(input)
    if not input or input == "" then
      return
    end
    local needle = "\\V" .. vim.fn.escape(input, "\\")
    vim.fn.setreg("/", needle)
    vim.opt.hlsearch = true
    local found = vim.fn.search(needle, "w")
    if found == 0 then
      notify("not found: " .. input, vim.log.levels.WARN)
    end
  end)
end

function M.find_next()
  pcall(vim.cmd, "normal! n")
end

function M.find_prev()
  pcall(vim.cmd, "normal! N")
end

function M.replace()
  vim.ui.input({ prompt = "Find: " }, function(find)
    if not find or find == "" then
      return
    end
    vim.ui.input({ prompt = "Replace with: " }, function(replacement)
      if replacement == nil then
        return
      end
      local pattern = vim.fn.escape(find, "/\\")
      local value = vim.fn.escape(replacement, "/\\&")
      local ok, err = pcall(vim.cmd, "silent! %s/" .. pattern .. "/" .. value .. "/g")
      if not ok then
        notify(tostring(err), vim.log.levels.ERROR)
      end
    end)
  end)
end

function M.goto_line()
  vim.ui.input({
    prompt = "Go to line or file:line:col: ",
  }, function(input)
    if not input or input == "" then
      return
    end

    local file, line, column = input:match("^(.-):(%d+):(%d+)$")
    if not file then
      file, line = input:match("^(.-):(%d+)$")
    end
    if not file then
      line = input:match("^(%d+)$")
    end
    if not line then
      notify("not a line or file location: " .. input, vim.log.levels.WARN)
      return
    end

    line = tonumber(line)
    column = tonumber(column) or 1

    if file and file ~= "" then
      local expanded = vim.fn.expand(file)
      if vim.fn.filereadable(expanded) == 0 then
        notify("cannot read file: " .. file, vim.log.levels.WARN)
        return
      end
      M.open_path(expanded)
    end

    local line_count = vim.api.nvim_buf_line_count(0)
    line = math.max(1, math.min(line, line_count))
    local line_text = vim.fn.getline(line)
    local col0 = math.max(0, math.min(column - 1, #line_text))
    vim.api.nvim_win_set_cursor(0, { line, col0 })
    vim.cmd("normal! zv")
  end)
end

function M.select_next()
  local word = vim.fn.expand("<cword>")
  if word == "" then
    return
  end
  local needle = "\\<" .. vim.fn.escape(word, "\\") .. "\\>"
  vim.fn.setreg("/", needle)
  vim.opt.hlsearch = true
  pcall(vim.cmd, "normal! gn")
end

function M.toggle_comment()
  local commentstring = vim.bo.commentstring
  if commentstring == "" or not commentstring:find("%%s") then
    notify("no line comment string for " .. vim.bo.filetype, vim.log.levels.WARN)
    return
  end

  local prefix = commentstring:match("^(.-)%%s") or ""
  local suffix = commentstring:match("%%s(.*)$") or ""
  local line = vim.fn.getline(".")

  if line:sub(1, #prefix) == prefix and (suffix == "" or line:sub(-#suffix) == suffix) then
    local replacement = line:sub(#prefix + 1)
    if suffix ~= "" then
      replacement = replacement:sub(1, #replacement - #suffix)
    end
    vim.fn.setline(".", replacement)
  else
    vim.fn.setline(".", prefix .. line .. suffix)
  end
end

function M.duplicate_line()
  pcall(vim.cmd, "normal! yyp")
end

function M.move_line(direction)
  if direction == "up" then
    pcall(vim.cmd, "normal! ddkP")
  elseif direction == "down" then
    pcall(vim.cmd, "normal! ddp")
  else
    notify("move direction must be up or down", vim.log.levels.WARN)
    return
  end
  pcall(vim.cmd, "normal! ==")
end

function M.move_line_command(opts)
  M.move_line(opts.args)
end

function M.select_all()
  pcall(vim.cmd, "normal! ggVG")
end

function M.undo()
  pcall(vim.cmd, "undo")
end

function M.redo()
  pcall(vim.cmd, "redo")
end

function M.find_file()
  local files = list_files()
  if #files == 0 then
    notify("no files found in " .. vim.fn.getcwd(), vim.log.levels.WARN)
    return
  end
  vim.ui.select(files, {
    prompt = "Open file",
    format_item = function(item)
      return item
    end,
  }, function(choice)
    if choice then
      M.open_path(choice)
    end
  end)
end

function M.command_palette()
  local commands = vim.fn.getcompletion("", "command")
  if #commands == 0 then
    notify("no commands available", vim.log.levels.WARN)
    return
  end
  vim.ui.select(commands, {
    prompt = "Command",
    format_item = function(item)
      return item
    end,
  }, function(choice)
    if choice then
      vim.cmd(choice)
    end
  end)
end

function M.help()
  local lines = {
    "# subcli",
    "",
    "Sublime-like modal editing on Neovim.",
    "",
    "## Modes",
    "",
    "Normal, insert, visual, and command modes are standard VI modal editing.",
    "",
    "## Default keybindings",
    "",
    "| Key | Action |",
    "| --- | --- |",
    "| Ctrl+S | Save |",
    "| Ctrl+Shift+S | Save As |",
    "| Ctrl+O | Open file |",
    "| Ctrl+N | New file |",
    "| Ctrl+W | Close buffer |",
    "| Ctrl+Shift+W | Close window |",
    "| Ctrl+P | Open file by name |",
    "| Ctrl+Shift+P | Command palette |",
    "| Ctrl+F | Find |",
    "| Ctrl+H | Replace |",
    "| Ctrl+G | Go to line or file:line:col |",
    "| Ctrl+D | Select next occurrence |",
    "| Ctrl+/ | Toggle line comment |",
    "| Ctrl+C | Copy line or selection |",
    "| Ctrl+X | Cut line or selection |",
    "| Ctrl+V | Paste |",
    "| Ctrl+Z | Undo |",
    "| Ctrl+Y | Redo |",
    "| Ctrl+A | Select all |",
    "| Ctrl+Shift+D | Duplicate line |",
    "| Ctrl+Shift+K | Delete line |",
    "| Alt+Up/Down | Move line up or down |",
    "| Alt+H/J/K/L | Move between windows |",
    "| Alt+V | Split vertically |",
    "| Alt+S | Split horizontally |",
    "| F1 | This help |",
    "",
    "## Sublime packages",
    "",
    "Settings, keymaps, snippets, and syntax file-extension associations are read from Sublime package roots. Use `:SubcliPackages` to inspect what was loaded and `:SubcliHealth` for a summary.",
    "",
    "Press q to close.",
  }
  scratch("subcli://help", lines, "markdown")
end

function M.health()
  local sublime = require("subcli.sublime")
  local stats = sublime.stats
  local roots = sublime.package_roots()
  local lines = {
    "# subcli health",
    "",
    "- Neovim: " .. tostring(vim.version()),
    "- subcli: " .. tostring(vim.g.subcli_version),
    "- Root: " .. tostring(vim.g.subcli_root),
    "- Package roots: " .. tostring(#roots),
    "- Packages: " .. tostring(stats.packages),
    "- Settings: " .. tostring(stats.settings),
    "- Keymaps: " .. tostring(stats.keymaps),
    "- Snippets: " .. tostring(stats.snippets),
    "- Syntax extensions: " .. tostring(stats.syntaxes),
    "- Skipped keybindings: " .. tostring(stats.skipped),
    "",
    "## Package roots",
    "",
  }
  for _, root in ipairs(roots) do
    table.insert(lines, "- `" .. root .. "`")
  end
  table.insert(lines, "")
  table.insert(lines, "## Warnings")
  table.insert(lines, "")
  if #sublime.warnings == 0 then
    table.insert(lines, "None.")
  else
    for _, warning in ipairs(sublime.warnings) do
      table.insert(lines, "- " .. warning)
    end
  end
  table.insert(lines, "")
  table.insert(lines, "Press q to close.")
  scratch("subcli://health", lines, "markdown")
end

function M.packages()
  local sublime = require("subcli.sublime")
  local lines = {
    "# Sublime packages",
    "",
  }
  if #sublime.packages == 0 then
    table.insert(lines, "No packages found.")
  else
    for _, package in ipairs(sublime.packages) do
      table.insert(lines, "- **" .. package.name .. "** — `" .. package.path .. "`")
    end
  end
  table.insert(lines, "")
  table.insert(lines, "## Warnings")
  table.insert(lines, "")
  if #sublime.warnings == 0 then
    table.insert(lines, "None.")
  else
    for _, warning in ipairs(sublime.warnings) do
      table.insert(lines, "- " .. warning)
    end
  end
  table.insert(lines, "")
  table.insert(lines, "Press q to close.")
  scratch("subcli://packages", lines, "markdown")
end

function M.reload()
  for _, name in ipairs({
    "subcli.options",
    "subcli.theme",
    "subcli.commands",
    "subcli.keymaps",
    "subcli.sublime",
  }) do
    package.loaded[name] = nil
  end
  vim.cmd("source " .. vim.fn.fnameescape(vim.g.subcli_root .. "/init.lua"))
  notify("reloaded")
end

local specs = {
  { "SubcliWrite", M.write, { desc = "Write current buffer" } },
  { "SubcliSaveAs", M.save_as, { desc = "Save current buffer as" } },
  { "SubcliOpen", M.open_command, { nargs = "?", desc = "Open a file" } },
  { "SubcliNewFile", M.new_file, { desc = "New file" } },
  { "SubcliCloseFile", M.close_file, { desc = "Close current buffer" } },
  { "SubcliCloseAll", M.close_all, { desc = "Close all buffers" } },
  { "SubcliFind", M.find, { desc = "Find" } },
  { "SubcliFindNext", M.find_next, { desc = "Find next" } },
  { "SubcliFindPrev", M.find_prev, { desc = "Find previous" } },
  { "SubcliReplace", M.replace, { desc = "Replace" } },
  { "SubcliGoto", M.goto_line, { desc = "Go to line or file location" } },
  { "SubcliSelectNext", M.select_next, { desc = "Select next occurrence" } },
  { "SubcliToggleComment", M.toggle_comment, { desc = "Toggle line comment" } },
  { "SubcliDuplicateLine", M.duplicate_line, { desc = "Duplicate current line" } },
  { "SubcliMoveLine", M.move_line_command, { nargs = "?", desc = "Move current line up or down" } },
  { "SubcliSelectAll", M.select_all, { desc = "Select all" } },
  { "SubcliUndo", M.undo, { desc = "Undo" } },
  { "SubcliRedo", M.redo, { desc = "Redo" } },
  { "SubcliFindFile", M.find_file, { desc = "Open file by name" } },
  { "SubcliCommands", M.command_palette, { desc = "Command palette" } },
  { "SubcliHelp", M.help, { desc = "subcli help" } },
  { "SubcliHealth", M.health, { desc = "subcli health" } },
  { "SubcliPackages", M.packages, { desc = "Sublime package status" } },
  { "SubcliReload", M.reload, { desc = "Reload subcli configuration" } },
}

function M.setup()
  for _, spec in ipairs(specs) do
    local opts = vim.tbl_extend("force", { force = true }, spec[3] or {})
    vim.api.nvim_create_user_command(spec[1], spec[2], opts)
  end
end

return M
