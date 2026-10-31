#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
export SUBCLI_SUBLIME_PACKAGES="$root/tests/fixtures/Packages"

nvim --headless -u "$root/init.lua" \
  +'lua local s = require("subcli.sublime"); assert(s.stats.packages == 1, "package discovery"); assert(s.stats.settings >= 1, "settings"); assert(vim.o.shiftwidth == 2, "tab_size"); assert(vim.o.expandtab == true, "translate_tabs_to_spaces"); assert(vim.o.colorcolumn == "80", "rulers"); assert(vim.g.subcli_trim_trailing_whitespace == true, "trim on save"); assert(s.stats.snippets >= 1, "snippets"); assert(s.stats.syntaxes >= 1, "syntax extensions"); assert(vim.filetype.match({ filename = "sample.foo" }) == "python", "filetype association"); assert(vim.fn.maparg("<F9>", "n") ~= "", "sublime keymap"); assert(vim.fn.maparg("<C-s>", "n") ~= "", "default keymap"); print("subcli tests passed\n")' \
  +qa

nvim --headless -u "$root/init.lua" \
  +'set filetype=python' \
  +'call setline(1, "hello ")' \
  +'lua local s = require("subcli.sublime"); if vim.snippet and vim.snippet.expand then vim.api.nvim_win_set_cursor(0, { 1, 5 }); assert(s.expand_snippet(), "snippet expansion"); assert(vim.api.nvim_get_current_line() == "Hello world ", "snippet content"); print("snippet expansion passed\n") else print("snippet expansion skipped on this Neovim\n") end' \
  +qa!
