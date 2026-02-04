@echo off
setlocal
set PATH=C:\programs\nodejs;%PATH%
cd /d "%~dp0"
REM Visible Edge window + slow motion so you can watch interactions
npm run test:visual:headed:slow
