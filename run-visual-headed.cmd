@echo off
setlocal
set PATH=C:\programs\nodejs;%PATH%
cd /d "%~dp0"
REM Runs Playwright in a visible Edge window, one worker for stability
npm run test:visual:headed
