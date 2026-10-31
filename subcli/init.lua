local source = debug.getinfo(1, "S").source
local root = vim.fn.fnamemodify(source:sub(2), ":p:h")

local function joinpath(...)
  if vim.fs and vim.fs.joinpath then
    return vim.fs.joinpath(...)
  end
  return table.concat({ ... }, package.config:sub(1, 1))
end

vim.g.subcli_root = root
vim.g.subcli_version = "0.1.0"
vim.g.mapleader = " "
vim.g.maplocalleader = " "
vim.opt.runtimepath:prepend(root)

require("subcli.options").setup()
require("subcli.theme").setup()
require("subcli.commands").setup()
require("subcli.keymaps").setup()
require("subcli.sublime").setup()

vim.keymap.set('n', '<leader>e', function() vim.cmd('Lexplore') end, { silent=true, desc='Toggle file explorer' })
vim.api.nvim_create_autocmd('VimEnter', {callback = function()
  local arg = vim.fn.argv(0)
  if arg ~= '' and vim.fn.isdirectory(arg) == 1 then
    vim.cmd('Lexplore')
  end
end})
local lazyvim_dir = vim.fn.expand('~/.config/subcli/lazyvim')
if vim.fn.isdirectory(lazyvim_dir) == 1 then
  vim.opt.rtp:append(lazyvim_dir)
  pcall(require, 'lazyvim')
end

local user_config = vim.env.SUBCLI_USER_CONFIG
if not user_config and vim.env.NVIM_APPNAME == "subcli" then
  user_config = joinpath(vim.fn.stdpath("config"), "user.lua")
end
if user_config and vim.fn.filereadable(user_config) == 1 then
  vim.cmd.source(user_config)
end
