local M = {}

local function set(group, value)
  vim.api.nvim_set_hl(0, group, value)
end

function M.apply()
  set("Normal", { bg = "#1e1f22", fg = "#d4d4d4" })
  set("NormalNC", { bg = "#1e1f22", fg = "#d4d4d4" })
  set("NormalFloat", { bg = "#2b2d30", fg = "#d4d4d4" })
  set("FloatBorder", { bg = "#2b2d30", fg = "#3c3f41" })
  set("CursorLine", { bg = "#2a2d31" })
  set("CursorColumn", { bg = "#2a2d31" })
  set("LineNr", { fg = "#6b6f76" })
  set("CursorLineNr", { fg = "#d7ba7d", bold = true })
  set("SignColumn", { bg = "#1e1f22", fg = "#6b6f76" })
  set("Visual", { bg = "#264f78" })
  set("Search", { bg = "#623315", fg = "#ffffff" })
  set("IncSearch", { bg = "#d7ba7d", fg = "#1e1f22" })
  set("CurSearch", { bg = "#d7ba7d", fg = "#1e1f22" })
  set("Pmenu", { bg = "#2b2d30", fg = "#d4d4d4" })
  set("PmenuSel", { bg = "#264f78", fg = "#ffffff" })
  set("PmenuSbar", { bg = "#2b2d30" })
  set("PmenuThumb", { bg = "#4b4f55" })
  set("StatusLine", { bg = "#2b2d30", fg = "#d4d4d4" })
  set("StatusLineNC", { bg = "#1e1f22", fg = "#6b6f76" })
  set("WinSeparator", { fg = "#3c3f41" })
  set("VertSplit", { fg = "#3c3f41" })
  set("TabLine", { bg = "#1e1f22", fg = "#6b6f76" })
  set("TabLineSel", { bg = "#2b2d30", fg = "#d4d4d4" })
  set("TabLineFill", { bg = "#1e1f22" })
  set("Comment", { fg = "#6a9955", italic = true })
  set("String", { fg = "#ce9178" })
  set("Character", { fg = "#ce9178" })
  set("Number", { fg = "#b5cea8" })
  set("Boolean", { fg = "#569cd6" })
  set("Constant", { fg = "#b5cea8" })
  set("Identifier", { fg = "#9cdcfe" })
  set("Function", { fg = "#dcdcaa" })
  set("Statement", { fg = "#c586c0" })
  set("Keyword", { fg = "#c586c0" })
  set("Conditional", { fg = "#c586c0" })
  set("Repeat", { fg = "#c586c0" })
  set("Operator", { fg = "#d4d4d4" })
  set("Type", { fg = "#4ec9b0" })
  set("StorageClass", { fg = "#569cd6" })
  set("Structure", { fg = "#4ec9b0" })
  set("PreProc", { fg = "#c586c0" })
  set("Special", { fg = "#d7ba7d" })
  set("Delimiter", { fg = "#d4d4d4" })
  set("Todo", { bg = "#d7ba7d", fg = "#1e1f22", bold = true })
  set("DiagnosticError", { fg = "#f44747" })
  set("DiagnosticWarn", { fg = "#d7ba7d" })
  set("DiagnosticInfo", { fg = "#4fc1ff" })
  set("DiagnosticHint", { fg = "#4ec9b0" })
  set("DiffAdd", { bg = "#1f3d2b" })
  set("DiffChange", { bg = "#2b2d30" })
  set("DiffDelete", { bg = "#4b1f1f" })
  set("DiffText", { bg = "#264f78" })
end

function M.setup()
  vim.o.background = "dark"
  pcall(vim.cmd, "colorscheme habamax")
  M.apply()

  local group = vim.api.nvim_create_augroup("SubcliTheme", { clear = true })
  vim.api.nvim_create_autocmd("ColorScheme", {
    group = group,
    callback = function()
      M.apply()
    end,
  })

  if vim.hl and vim.hl.on_yank then
    vim.hl.on_yank({ higroup = "IncSearch", timeout = 150 })
  end
end

return M
