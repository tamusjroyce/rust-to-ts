@echo off
setlocal enabledelayedexpansion

REM Simple wrapper to run the PowerShell pipeline
pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0run-all.ps1"

endlocal
