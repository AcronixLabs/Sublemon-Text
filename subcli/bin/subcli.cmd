@echo off
setlocal

set "HERE=%~dp0"
set "CONFIG="

if defined SUBCLI_SHARE set "CONFIG=%SUBCLI_SHARE%"
if not defined CONFIG if exist "%HERE%..\init.lua" set "CONFIG=%HERE%.."
if not defined CONFIG if exist "%HERE%..\share\subcli\init.lua" set "CONFIG=%HERE%..\share\subcli"
if not defined CONFIG if exist "%ProgramFiles%\subcli\init.lua" set "CONFIG=%ProgramFiles%\subcli"

if not defined CONFIG (
  echo subcli: cannot locate init.lua; set SUBCLI_SHARE 1>&2
  exit /b 1
)

set "NVIM_APPNAME=subcli"
if not defined NVIM set "NVIM=nvim"

"%NVIM%" -u "%CONFIG%\init.lua" %*
