local M = {}

local function cmd(name)
  return "<Cmd>" .. name .. "<CR>"
end

local function map(modes, lhs, rhs, desc)
  local ok, err = pcall(vim.keymap.set, modes, lhs, rhs, {
    silent = true,
    desc = desc,
  })
  if not ok then
    vim.notify("subcli: cannot map " .. lhs .. ": " .. tostring(err), vim.log.levels.DEBUG)
  end
end

function M.setup()
  map({ "n", "v" }, "<C-s>", cmd("SubcliWrite"), "Save")
  map("i", "<C-s>", cmd("SubcliWrite"), "Save")
  map({ "n", "v" }, "<C-S-s>", cmd("SubcliSaveAs"), "Save As")
  map("i", "<C-S-s>", cmd("SubcliSaveAs"), "Save As")

  map("n", "<C-o>", cmd("SubcliOpen"), "Open file")
  map("i", "<C-o>", "<Esc>" .. cmd("SubcliOpen"), "Open file")
  map("n", "<C-n>", cmd("SubcliNewFile"), "New file")
  map("n", "<C-w>", cmd("SubcliCloseFile"), "Close buffer")
  map("n", "<C-S-w>", "<Cmd>close<CR>", "Close window")
  map("n", "<C-Tab>", "<Cmd>bnext<CR>", "Next buffer")
  map("n", "<C-S-Tab>", "<Cmd>bprevious<CR>", "Previous buffer")

  map("n", "<C-p>", cmd("SubcliFindFile"), "Open file by name")
  map("n", "<C-S-p>", cmd("SubcliCommands"), "Command palette")
  map("n", "<C-f>", cmd("SubcliFind"), "Find")
  map("n", "<C-h>", cmd("SubcliReplace"), "Replace")
  map("n", "<C-g>", cmd("SubcliGoto"), "Go to line")
  map("n", "<C-d>", cmd("SubcliSelectNext"), "Select next occurrence")

  map("n", "<C-/>", cmd("SubcliToggleComment"), "Toggle comment")
  map("v", "<C-/>", "gc", "Toggle comment")
  map("n", "<C-S-d>", cmd("SubcliDuplicateLine"), "Duplicate line")
  map("n", "<C-S-k>", "dd", "Delete line")
  map("v", "<C-S-k>", "d", "Delete selection")

  map("n", "<C-c>", '"+yy', "Copy line")
  map("v", "<C-c>", '"+y', "Copy selection")
  map("n", "<C-x>", '"+dd', "Cut line")
  map("v", "<C-x>", '"+d', "Cut selection")
  map({ "n", "v" }, "<C-v>", '"+p', "Paste")
  map({ "n", "v" }, "<C-S-v>", '"+p', "Paste")
  map({ "n", "v" }, "<C-z>", cmd("SubcliUndo"), "Undo")
  map({ "n", "v" }, "<C-y>", cmd("SubcliRedo"), "Redo")
  map({ "n", "v" }, "<C-a>", "ggVG", "Select all")

  map("n", "<M-Up>", cmd("SubcliMoveLine up"), "Move line up")
  map("n", "<M-Down>", cmd("SubcliMoveLine down"), "Move line down")
  map("v", "<M-Up>", ":m '<-2<CR>gv=gv", "Move selection up")
  map("v", "<M-Down>", ":m '>+1<CR>gv=gv", "Move selection down")

  map("n", "<M-h>", "<C-w>h", "Window left")
  map("n", "<M-j>", "<C-w>j", "Window down")
  map("n", "<M-k>", "<C-w>k", "Window up")
  map("n", "<M-l>", "<C-w>l", "Window right")
  map("n", "<M-v>", "<Cmd>vsplit<CR>", "Split vertically")
  map("n", "<M-s>", "<Cmd>split<CR>", "Split horizontally")

  map("n", "<F1>", cmd("SubcliHelp"), "subcli help")
  map("n", "<C-S-r>", cmd("SubcliReload"), "Reload subcli")
  map("n", "<C-S-e>", cmd("SubcliPackages"), "Sublime package status")
end

return M
