local M = {}

function M.setup()
  local opt = vim.opt

  opt.termguicolors = true
  opt.mouse = "a"
  opt.clipboard = "unnamedplus"
  opt.fileencoding = "utf-8"
  opt.fileformats = "unix,dos,mac"
  opt.bomb = false
  opt.fixendofline = true
  opt.endofline = true

  opt.number = true
  opt.relativenumber = false
  opt.numberwidth = 4
  opt.signcolumn = "yes"
  opt.cursorline = true
  opt.cursorcolumn = false
  opt.wrap = false
  opt.linebreak = true
  opt.breakindent = true
  opt.showbreak = "  "
  opt.scrolloff = 5
  opt.sidescrolloff = 8
  opt.colorcolumn = ""
  opt.list = false
  opt.listchars = { tab = "»·", trail = "·", nbsp = "·" }
  opt.fillchars = { eob = " " }

  opt.expandtab = true
  opt.shiftwidth = 4
  opt.tabstop = 4
  opt.softtabstop = 4
  opt.shiftround = true
  opt.autoindent = true
  opt.smartindent = true
  opt.copyindent = true
  opt.preserveindent = true
  opt.textwidth = 0
  opt.virtualedit = "block"
  opt.whichwrap = "b,s,h,l,<,>,[,]"

  opt.ignorecase = true
  opt.smartcase = true
  opt.incsearch = true
  opt.hlsearch = true
  opt.inccommand = "split"

  opt.updatetime = 300
  opt.timeoutlen = 500
  opt.ttimeoutlen = 10
  opt.history = 1000
  opt.undolevels = 10000
  opt.undofile = true
  opt.swapfile = false
  opt.backup = false
  opt.writebackup = false
  opt.hidden = true
  opt.confirm = true

  opt.completeopt = { "menuone", "noselect" }
  opt.pumheight = 12
  opt.wildmenu = true
  opt.wildmode = "longest:full,full"
  opt.cmdheight = 1
  opt.laststatus = 2
  opt.statusline = "%f %m %= %y %l:%c  %p%%"
  opt.showcmd = true
  opt.showmode = true
  opt.ruler = true
  opt.title = true
  opt.titlestring = "subcli - %f"
  opt.errorbells = false
  opt.visualbell = false
  opt.belloff = "all"
  opt.synmaxcol = 300
  opt.exrc = false

  local undodir = vim.fn.stdpath("state") .. "/undo"
  vim.fn.mkdir(undodir, "p")
  opt.undodir = undodir

  vim.g.subcli_trim_trailing_whitespace = false
  vim.g.subcli_ensure_newline = false
  vim.g.subcli_load_sublime_keymaps = true

  local group = vim.api.nvim_create_augroup("SubcliOptions", { clear = true })
  vim.api.nvim_create_autocmd("BufReadPost", {
    group = group,
    callback = function()
      local mark = vim.api.nvim_buf_get_mark(0, '"')
      local line_count = vim.api.nvim_buf_line_count(0)
      if mark[1] > 0 and mark[1] <= line_count then
        pcall(vim.api.nvim_win_set_cursor, 0, mark)
      end
    end,
  })
end

return M
