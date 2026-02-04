@echo off
setlocal
set PATH=C:\programs\nodejs;%PATH%
cd /d "%~dp0"
REM Playwright UI runner (lets you click tests / watch / debug)
npm run test:visual:ui
