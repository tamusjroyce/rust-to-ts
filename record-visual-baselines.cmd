@echo off
setlocal
set PATH=C:\programs\nodejs;%PATH%
cd /d "%~dp0"
npm run test:visual:update
