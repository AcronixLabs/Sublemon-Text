local M = {}

M.stats = {
  packages = 0,
  settings = 0,
  keymaps = 0,
  snippets = 0,
  syntaxes = 0,
  skipped = 0,
}
M.packages = {}
M.snippets = {}
M.warnings = {}
M.ignored_packages = {}

local warned = {}
local builtin_filetypes = {}

local function warn(message)
  if warned[message] then
    return
  end
  warned[message] = true
  table.insert(M.warnings, message)
end

local function path_list_separator()
  if package.config:sub(1, 1) == "\\" then
    return ";"
  end
  return ":"
end

local function joinpath(...)
  if vim.fs and vim.fs.joinpath then
    return vim.fs.joinpath(...)
  end
  return table.concat({ ... }, package.config:sub(1, 1))
end

local function read_file(path)
  local handle = io.open(path, "rb")
  if not handle then
    return nil
  end
  local data = handle:read("*a")
  handle:close()
  if not data then
    return nil
  end
  return data:gsub("^\239\187\191", "")
end

local function strip_comments(text)
  local output = {}
  local index = 1
  local length = #text
  local in_string = false
  local escaped = false

  while index <= length do
    local char = text:sub(index, index)
    if in_string then
      table.insert(output, char)
      if escaped then
        escaped = false
      elseif char == "\\" then
        escaped = true
      elseif char == '"' then
        in_string = false
      end
      index = index + 1
    elseif char == '"' then
      in_string = true
      table.insert(output, char)
      index = index + 1
    elseif char == "/" and text:sub(index + 1, index + 1) == "/" then
      while index <= length and text:sub(index, index) ~= "\n" do
        index = index + 1
      end
    elseif char == "/" and text:sub(index + 1, index + 1) == "*" then
      index = index + 2
      while index <= length and not (text:sub(index, index) == "*" and text:sub(index + 1, index + 1) == "/") do
        index = index + 1
      end
      index = index + 2
    else
      table.insert(output, char)
      index = index + 1
    end
  end

  return table.concat(output)
end

local function strip_trailing_commas(text)
  local output = {}
  local index = 1
  local length = #text
  local in_string = false
  local escaped = false

  while index <= length do
    local char = text:sub(index, index)
    if in_string then
      table.insert(output, char)
      if escaped then
        escaped = false
      elseif char == "\\" then
        escaped = true
      elseif char == '"' then
        in_string = false
      end
      index = index + 1
    elseif char == '"' then
      in_string = true
      table.insert(output, char)
      index = index + 1
    elseif char == "," then
      local lookahead = index + 1
      while lookahead <= length and text:sub(lookahead, lookahead):match("%s") do
        lookahead = lookahead + 1
      end
      local next_char = text:sub(lookahead, lookahead)
      if next_char ~= "}" and next_char ~= "]" then
        table.insert(output, char)
      end
      index = index + 1
    else
      table.insert(output, char)
      index = index + 1
    end
  end

  return table.concat(output)
end

local function decode_file(path)
  local text = read_file(path)
  if not text then
    warn("cannot read " .. path)
    return nil
  end
  local cleaned = strip_trailing_commas(strip_comments(text))
  local ok, data = pcall(vim.json.decode, cleaned)
  if not ok then
    warn("invalid JSON in " .. path)
    return nil
  end
  return data
end

local function find_files(directory, extension)
  local files = {}
  local seen = {}
  for _, pattern in ipairs({ "*." .. extension, "**/*." .. extension }) do
    local ok, matches = pcall(vim.fn.globpath, directory, pattern, false, true)
    if ok and type(matches) == "table" then
      for _, path in ipairs(matches) do
        if not seen[path] then
          seen[path] = true
          table.insert(files, path)
        end
      end
    end
  end
  table.sort(files)
  return files
end

local function default_roots()
  local roots = {}
  local seen = {}
  local function add(path)
    if path and path ~= "" and not seen[path] then
      seen[path] = true
      table.insert(roots, path)
    end
  end

  local home = vim.env.HOME or vim.fn.expand("~")
  local xdg = vim.env.XDG_CONFIG_HOME
  if xdg and xdg ~= "" then
    add(joinpath(xdg, "sublime-text", "Packages"))
  end
  add(joinpath(home, ".config", "sublime-text", "Packages"))
  add(joinpath(home, ".config", "sublime-text-4", "Packages"))
  add(joinpath(home, ".config", "sublime-text-3", "Packages"))
  add(joinpath(home, "Library", "Application Support", "Sublime Text", "Packages"))
  add(joinpath(home, "Library", "Application Support", "Sublime Text 3", "Packages"))

  local appdata = vim.env.APPDATA
  if appdata and appdata ~= "" then
    add(joinpath(appdata, "Sublime Text", "Packages"))
    add(joinpath(appdata, "Sublime Text 3", "Packages"))
  end

  return roots
end

function M.package_roots()
  local override = vim.env.SUBCLI_SUBLIME_PACKAGES
  if override and override ~= "" then
    local roots = {}
    for _, path in ipairs(vim.split(override, path_list_separator(), { plain = true })) do
      if path ~= "" then
        table.insert(roots, path)
      end
    end
    return roots
  end
  return default_roots()
end

local function load_packages()
  local packages = {}
  local seen = {}
  for _, root in ipairs(M.package_roots()) do
    if vim.fn.isdirectory(root) == 1 then
      local ok, matches = pcall(vim.fn.globpath, root, "*", false, true)
      if ok and type(matches) == "table" then
        for _, path in ipairs(matches) do
          if vim.fn.isdirectory(path) == 1 and not seen[path] then
            seen[path] = true
            table.insert(packages, {
              name = vim.fn.fnamemodify(path, ":t"),
              path = path,
            })
          end
        end
      end
    end
  end

  table.sort(packages, function(left, right)
    local left_rank = left.name == "Default" and 0 or (left.name == "User" and 2 or 1)
    local right_rank = right.name == "Default" and 0 or (right.name == "User" and 2 or 1)
    if left_rank ~= right_rank then
      return left_rank < right_rank
    end
    return left.name:lower() < right.name:lower()
  end)

  M.packages = packages
  M.stats.packages = #packages
end

local settings = {
  tab_size = function(value)
    if type(value) == "number" then
      vim.opt.shiftwidth = value
      vim.opt.tabstop = value
      vim.opt.softtabstop = value
    end
  end,
  translate_tabs_to_spaces = function(value)
    vim.opt.expandtab = value == true
  end,
  word_wrap = function(value)
    vim.opt.wrap = value == true
  end,
  rulers = function(value)
    if type(value) == "table" then
      local columns = {}
      for _, ruler in ipairs(value) do
        if type(ruler) == "number" then
          table.insert(columns, tostring(ruler))
        end
      end
      vim.opt.colorcolumn = table.concat(columns, ",")
    elseif type(value) == "number" then
      vim.opt.colorcolumn = tostring(value)
    end
  end,
  line_numbers = function(value)
    vim.opt.number = value == true
  end,
  highlight_line = function(value)
    vim.opt.cursorline = value == true
  end,
  draw_white_space = function(value)
    vim.opt.list = value == "all" or value == "selection" or value == true
  end,
  trim_trailing_white_space_on_save = function(value)
    vim.g.subcli_trim_trailing_whitespace = value == true
  end,
  ensure_newline_at_eof_on_save = function(value)
    vim.g.subcli_ensure_newline = value == true
  end,
  auto_indent = function(value)
    vim.opt.autoindent = value == true
  end,
  smart_indent = function(value)
    vim.opt.smartindent = value == true
  end,
  detect_indentation = function(value)
    vim.g.subcli_detect_indentation = value == true
  end,
  tab_completion = function(value)
    vim.g.subcli_tab_completion = value == true
  end,
  shift_tab_unindent = function(value)
    vim.g.subcli_shift_tab_unindent = value == true
  end,
  scroll_past_end = function(value)
    if value == true then
      vim.opt.scrolloff = math.max(vim.o.scrolloff, 3)
    end
  end,
  show_definitions = function() end,
  ignored_packages = function(value)
    if type(value) == "table" then
      M.ignored_packages = {}
      for _, name in ipairs(value) do
        M.ignored_packages[name] = true
      end
    end
  end,
}

local function load_settings(package)
  local files = find_files(package.path, "sublime-settings")
  table.sort(files, function(left, right)
    local left_preferences = left:match("Preferences%.sublime%-settings$") ~= nil
    local right_preferences = right:match("Preferences%.sublime%-settings$") ~= nil
    if left_preferences ~= right_preferences then
      return not left_preferences
    end
    return left < right
  end)

  for _, path in ipairs(files) do
    local data = decode_file(path)
    if type(data) == "table" then
      M.stats.settings = M.stats.settings + 1
      for key, value in pairs(data) do
        local handler = settings[key]
        if handler then
          local ok, err = pcall(handler, value)
          if not ok then
            warn("setting " .. key .. " failed: " .. tostring(err))
          end
        end
      end
    end
  end
end

local key_aliases = {
  pageup = "PageUp",
  pagedown = "PageDown",
  page_up = "PageUp",
  page_down = "PageDown",
  home = "Home",
  ["end"] = "End",
  insert = "Insert",
  delete = "Del",
  backspace = "BS",
  enter = "CR",
  ["return"] = "CR",
  tab = "Tab",
  space = "Space",
  escape = "Esc",
  esc = "Esc",
  up = "Up",
  down = "Down",
  left = "Left",
  right = "Right",
  plus = "+",
  minus = "-",
  comma = ",",
  period = ".",
  slash = "/",
  backslash = "\\",
  semicolon = ";",
  quote = "'",
  grave = "`",
  bracketleft = "[",
  bracketright = "]",
  equal = "=",
}

local modifier_aliases = {
  ctrl = "C",
  control = "C",
  alt = "M",
  option = "M",
  shift = "S",
  super = "D",
  cmd = "D",
  command = "D",
  meta = "D",
}

local function split_key_spec(spec)
  local parts = {}
  local start = 1
  while true do
    local position = spec:find("+", start, true)
    if not position then
      table.insert(parts, spec:sub(start))
      break
    end
    table.insert(parts, spec:sub(start, position - 1))
    start = position + 1
    if start > #spec then
      table.insert(parts, "+")
      break
    end
  end
  return parts
end

local function normalize_keys(keys)
  if type(keys) ~= "table" or #keys == 0 then
    return nil
  end

  local parts = {}
  for _, item in ipairs(keys) do
    if type(item) ~= "string" then
      return nil
    end
    for _, part in ipairs(split_key_spec(item)) do
      part = vim.trim(part):lower()
      if part ~= "" then
        table.insert(parts, part)
      end
    end
  end

  if #parts == 0 then
    return nil
  end

  local key = table.remove(parts)
  if key == "" or modifier_aliases[key] then
    return nil
  end

  local modifiers = {}
  for _, part in ipairs(parts) do
    local modifier = modifier_aliases[part]
    if not modifier then
      return nil
    end
    modifiers[modifier] = true
  end

  if key_aliases[key] then
    key = key_aliases[key]
  elseif key:match("^f%d+$") then
    key = key:upper()
  elseif #key == 1 then
    key = key:lower()
  else
    key = key:sub(1, 1):upper() .. key:sub(2)
  end

  local ordered = {}
  for _, modifier in ipairs({ "C", "S", "M", "D" }) do
    if modifiers[modifier] then
      table.insert(ordered, modifier)
    end
  end

  local prefix = #ordered > 0 and table.concat(ordered, "-") .. "-" or ""
  return "<" .. prefix .. key .. ">"
end

local function command(name)
  return "<Cmd>" .. name .. "<CR>"
end

local function command_map(name, arguments)
  if type(arguments) ~= "table" then
    arguments = {}
  end

  local mappings = {
    save = { n = command("SubcliWrite"), i = command("SubcliWrite"), v = command("SubcliWrite") },
    save_all = { n = "<Cmd>wall<CR>" },
    save_as = { n = command("SubcliSaveAs"), i = command("SubcliSaveAs") },
    open_file = { n = command("SubcliOpen") },
    prompt_open_file = { n = command("SubcliOpen") },
    new_file = { n = command("SubcliNewFile") },
    close_file = { n = command("SubcliCloseFile") },
    close_all_files = { n = command("SubcliCloseAll") },
    revert = { n = "<Cmd>edit!<CR>" },
    find = { n = command("SubcliFind") },
    find_next = { n = command("SubcliFindNext") },
    find_prev = { n = command("SubcliFindPrev") },
    find_under_expand = { n = command("SubcliSelectNext") },
    replace = { n = command("SubcliReplace") },
    goto_line = { n = command("SubcliGoto") },
    toggle_comment = { n = command("SubcliToggleComment"), v = "gc" },
    duplicate_line = { n = command("SubcliDuplicateLine") },
    delete_line = { n = "dd", v = "d" },
    delete_word = { n = "dw" },
    select_all = { n = "ggVG", v = "ggVG" },
    undo = { n = command("SubcliUndo"), v = command("SubcliUndo") },
    redo = { n = command("SubcliRedo"), v = command("SubcliRedo") },
    copy = { n = '"+yy', v = '"+y' },
    cut = { n = '"+dd', v = '"+d' },
    paste = { n = '"+p', v = '"+p' },
    paste_and_indent = { n = '"+p', v = '"+p' },
    indent = { n = ">>", v = ">" },
    unindent = { n = "<<", v = "<" },
    reindent = { n = "==", v = "=" },
    join_lines = { n = "J", v = "J" },
  }

  if name == "move" then
    local direction = arguments.forward and "down" or "up"
    return { n = command("SubcliMoveLine " .. direction) }
  end

  if name == "show_overlay" then
    local overlay = arguments.overlay
    if overlay == "goto" then
      return { n = command("SubcliGoto") }
    elseif overlay == "command_palette" then
      return { n = command("SubcliCommands") }
    elseif overlay == "files" or overlay == "goto_symbol_in_project" then
      return { n = command("SubcliFindFile") }
    elseif overlay == "goto_definition" or overlay == "goto_symbol" then
      return { n = "gd" }
    end
    return nil
  end

  return mappings[name]
end

local function load_keymaps(package)
  if M.ignored_packages[package.name] or vim.g.subcli_load_sublime_keymaps == false then
    return
  end

  for _, path in ipairs(find_files(package.path, "sublime-keymap")) do
    local data = decode_file(path)
    if type(data) == "table" then
      for _, binding in ipairs(data) do
        if type(binding) == "table" and type(binding.keys) == "table" and type(binding.command) == "string" then
          local keys = normalize_keys(binding.keys)
          if keys then
            local mapping = command_map(binding.command, binding.args)
            if mapping then
              local applied = false
              for mode, rhs in pairs(mapping) do
                local ok = pcall(vim.keymap.set, mode, keys, rhs, {
                  silent = true,
                  desc = "Sublime: " .. binding.command,
                })
                if ok then
                  applied = true
                end
              end
              if applied then
                M.stats.keymaps = M.stats.keymaps + 1
              else
                M.stats.skipped = M.stats.skipped + 1
                warn("cannot map " .. keys .. " for " .. binding.command)
              end
            else
              M.stats.skipped = M.stats.skipped + 1
              warn("unsupported Sublime command: " .. binding.command)
            end
          else
            M.stats.skipped = M.stats.skipped + 1
            warn("unsupported Sublime keybinding in " .. path)
          end
        end
      end
    end
  end
end

local function xml_unescape(text)
  text = text:gsub("<!%[CDATA%[(.-)%]%]>", "%1")
  text = text:gsub("&#x(%x+);", function(value)
    return string.char(tonumber(value, 16))
  end)
  text = text:gsub("&#(%d+);", function(value)
    return string.char(tonumber(value))
  end)
  text = text:gsub("&lt;", "<")
  text = text:gsub("&gt;", ">")
  text = text:gsub("&quot;", '"')
  text = text:gsub("&apos;", "'")
  text = text:gsub("&amp;", "&")
  return text
end

local function xml_tag(text, name)
  local value = text:match("<" .. name .. ">(.-)</" .. name .. ">")
  if not value then
    return nil
  end
  return xml_unescape(value)
end

local function load_snippets(package)
  if M.ignored_packages[package.name] then
    return
  end

  for _, path in ipairs(find_files(package.path, "sublime-snippet")) do
    local text = read_file(path)
    if text then
      local trigger = xml_tag(text, "tabTrigger")
      local content = xml_tag(text, "content")
      if trigger and trigger ~= "" and content and content ~= "" then
        trigger = vim.trim(trigger)
        local scope = xml_tag(text, "scope")
        if scope then
          scope = vim.trim(scope)
        end
        M.snippets[trigger] = M.snippets[trigger] or {}
        table.insert(M.snippets[trigger], {
          content = content,
          scope = scope,
          package = package.name,
          path = path,
        })
        M.stats.snippets = M.stats.snippets + 1
      else
        warn("invalid snippet: " .. path)
      end
    end
  end
end

local filetype_aliases = {
  js = "javascript",
  jsx = "javascriptreact",
  ts = "typescript",
  tsx = "typescriptreact",
  py = "python",
  rb = "ruby",
  md = "markdown",
  sh = "sh",
  shell = "sh",
  bash = "bash",
  ["c++"] = "cpp",
  ["c#"] = "cs",
  ["objective-c"] = "objc",
  ["obj-c"] = "objc",
  html = "html",
  css = "css",
  json = "json",
  yaml = "yaml",
  yml = "yaml",
  xml = "xml",
  sql = "sql",
  php = "php",
  go = "go",
  rust = "rust",
  rs = "rust",
  java = "java",
  kotlin = "kotlin",
  swift = "swift",
  lua = "lua",
  perl = "perl",
  r = "r",
  tex = "tex",
  latex = "tex",
}

local function scope_filetype(scope)
  if not scope or scope == "" then
    return nil
  end
  scope = vim.trim(scope:gsub("%s+#.*$", "")):gsub("^['\"]", ""):gsub("['\"]$", "")

  local name
  local source = scope:match("^source%.(.+)$")
  if source then
    name = source
  else
    local text = scope:match("^text%.([%w_]+)")
    if text then
      name = text
    else
      name = scope:match("([^%.]+)$")
    end
  end

  if not name then
    return nil
  end
  name = name:lower():gsub("%s+", "")
  return filetype_aliases[name] or name
end

local function parse_syntax(text)
  local scope = text:match("[\r\n]scope:%s*([^\r\n]+)") or text:match("^scope:%s*([^\r\n]+)")
  local extensions = {}
  local in_extensions = false

  for line in text:gmatch("[^\r\n]+") do
    local trimmed = vim.trim(line)
    if trimmed:match("^file_extensions:") then
      in_extensions = true
      local inline = trimmed:match("^file_extensions:%s*%[(.-)%]")
      if inline then
        for extension in inline:gmatch("[^,%s]+") do
          table.insert(extensions, extension)
        end
        in_extensions = false
      end
    elseif in_extensions then
      local extension = trimmed:match("^%-%s*([^%s#]+)")
      if extension then
        table.insert(extensions, extension)
      elseif trimmed ~= "" and not trimmed:match("^#") then
        in_extensions = false
      end
    end
  end

  return scope, extensions
end

local function is_builtin_filetype(extension)
  if builtin_filetypes[extension] == nil then
    builtin_filetypes[extension] = vim.filetype.match({ filename = "subcli-probe." .. extension }) ~= nil
  end
  return builtin_filetypes[extension]
end

local function load_syntaxes(package)
  if M.ignored_packages[package.name] then
    return
  end

  for _, path in ipairs(find_files(package.path, "sublime-syntax")) do
    local text = read_file(path)
    if text then
      local scope, extensions = parse_syntax(text)
      local filetype = scope_filetype(scope)
      if filetype then
        for _, extension in ipairs(extensions) do
          extension = extension:gsub("^['\"]", ""):gsub("['\"]$", ""):gsub("^%.", ""):gsub("[^%w_%-%.]", "")
          if extension ~= "" and not is_builtin_filetype(extension) then
            local ok = pcall(vim.filetype.add, {
              extension = { [extension] = filetype },
            })
            if ok then
              M.stats.syntaxes = M.stats.syntaxes + 1
            else
              warn("cannot associate ." .. extension .. " with " .. filetype)
            end
          end
        end
      end
    end
  end
end

function M.load()
  M.stats = {
    packages = 0,
    settings = 0,
    keymaps = 0,
    snippets = 0,
    syntaxes = 0,
    skipped = 0,
  }
  M.packages = {}
  M.snippets = {}
  M.warnings = {}
  M.ignored_packages = {}
  warned = {}
  builtin_filetypes = {}

  load_packages()
  for _, package in ipairs(M.packages) do
    load_settings(package)
  end
  for _, package in ipairs(M.packages) do
    load_keymaps(package)
  end
  for _, package in ipairs(M.packages) do
    load_snippets(package)
  end
  for _, package in ipairs(M.packages) do
    load_syntaxes(package)
  end

  return M.stats
end

local function snippet_for(trigger, filetype)
  local snippets = M.snippets[trigger]
  if not snippets then
    return nil
  end

  local fallback
  for index = #snippets, 1, -1 do
    local snippet = snippets[index]
    local scope = snippet.scope
    if not scope or scope == "" then
      fallback = fallback or snippet
    elseif filetype and scope:find(filetype, 1, true) then
      return snippet
    end
  end
  return fallback
end

function M.expand_snippet()
  if not vim.snippet or not vim.snippet.expand then
    return false
  end

  local line = vim.api.nvim_get_current_line()
  local cursor = vim.api.nvim_win_get_cursor(0)
  local column = cursor[2]
  local before = line:sub(1, column)
  local trigger = before:match("([%w_]+)$")
  if not trigger then
    return false
  end

  local snippet = snippet_for(trigger, vim.bo.filetype)
  if not snippet then
    return false
  end

  local row = cursor[1] - 1
  local start_column = column - #trigger
  local ok, err = pcall(vim.api.nvim_buf_set_text, 0, row, start_column, row, column, { "" })
  if not ok then
    warn("cannot expand snippet: " .. tostring(err))
    return false
  end

  ok, err = pcall(vim.snippet.expand, snippet.content)
  if not ok then
    warn("cannot expand snippet: " .. tostring(err))
    return false
  end
  return true
end

function M.setup()
  M.load()

  local group = vim.api.nvim_create_augroup("SubcliSublime", { clear = true })
  vim.api.nvim_create_autocmd("BufWritePre", {
    group = group,
    callback = function(args)
      if not vim.g.subcli_trim_trailing_whitespace and not vim.g.subcli_ensure_newline then
        return
      end

      local lines = vim.api.nvim_buf_get_lines(args.buf, 0, -1, false)
      local changed = false

      if vim.g.subcli_trim_trailing_whitespace then
        for index, line in ipairs(lines) do
          local trimmed = line:gsub("%s+$", "")
          if trimmed ~= line then
            lines[index] = trimmed
            changed = true
          end
        end
      end

      if vim.g.subcli_ensure_newline and #lines > 0 and lines[#lines] ~= "" then
        table.insert(lines, "")
        changed = true
      end

      if changed then
        vim.api.nvim_buf_set_lines(args.buf, 0, -1, false, lines)
      end
    end,
  })

  if vim.g.subcli_tab_completion ~= false then
    vim.keymap.set("i", "<Tab>", function()
      if M.expand_snippet() then
        return
      end
      vim.api.nvim_feedkeys(vim.api.nvim_replace_termcodes("<Tab>", true, false, true), "n", false)
    end, {
      silent = true,
      desc = "Expand Sublime snippet or indent",
    })
  end
end

return M
