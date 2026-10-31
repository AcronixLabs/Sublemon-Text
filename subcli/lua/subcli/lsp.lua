local ok, lsp = pcall(require, 'lspconfig')
if not ok then
  vim.notify('subcli LSP: lspconfig not available, skipping LSP setup', vim.log.levels.WARN)
  return {}
end

-- Basic LSP server setup (pyright and tsserver)
local servers = {
  pyright = {},
  tsserver = {},
}

for name, opts in pairs(servers) do
  local ok2, err = pcall(function() lsp[name].setup(opts) end)
  if not ok2 then
    vim.notify('subcli LSP setup error for ' .. name .. ': ' .. tostring(err), vim.log.levels.ERROR)
  end
end

return {}