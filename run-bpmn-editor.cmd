@echo off
setlocal

REM Launch the Tauri BPMN editor from repo root.
REM Prereqs: Node.js + npm installed.

REM If VS Code was opened before Node was installed, PATH may be stale.
REM If Node is installed under C:\programs\nodejs, prepend it for this process.
where node >nul 2>nul
if not errorlevel 1 goto :node_ok

if not "%NODEJS_DIR%"=="" if exist "%NODEJS_DIR%\node.exe" set "PATH=%NODEJS_DIR%;%PATH%"
if exist "C:\programs\nodejs\node.exe" set "PATH=C:\programs\nodejs;%PATH%"

:node_ok

where node >nul 2>nul
if errorlevel 1 (
  echo Node.js is not installed or not on PATH.
  echo Expected it on PATH, or at C:\programs\nodejs\node.exe.
  exit /b 1
)

where npm >nul 2>nul
if errorlevel 1 (
  echo npm is not installed or not on PATH.
  echo Expected it alongside Node.js.
  exit /b 1
)

cd /d "%~dp0tools\bpmn-editor-tauri" || exit /b 1

echo Starting BPMN editor (Tauri)...

set NEED_NPM_INSTALL=
if not exist "node_modules" set NEED_NPM_INSTALL=1
if not exist "node_modules\.bin\tauri.cmd" set NEED_NPM_INSTALL=1
if not exist "node_modules\bpmn-auto-layout" set NEED_NPM_INSTALL=1

if not defined NEED_NPM_INSTALL goto :run_dev

echo Installing frontend dependencies (npm install)...
call npm install
if errorlevel 1 goto :npm_failed

:run_dev
call npm run dev
exit /b %errorlevel%

:npm_failed
echo npm install failed.
exit /b 1
