@echo off
setlocal ENABLEDELAYEDEXPANSION

REM Change to repo root (folder containing this script)
cd /d "%~dp0"

echo === Building Rust binaries (rust-to-ts, tester) ===
cargo build --bins
if errorlevel 1 goto :error

REM List of example subfolders under Examples\
set EXAMPLES=HelloWorld NeuralNetwork src

echo.
echo === Converting Rust -> TypeScript for all examples ===
for %%E in (%EXAMPLES%) do (
  echo.
  echo --- Converting Examples\%%E ---
  cargo run --bin rust-to-ts -- "Examples\%%E"
  if errorlevel 1 goto :error
)

echo.
echo === Running tester for all examples ===
for %%E in (%EXAMPLES%) do (
  echo.
  echo --- Testing Examples\%%E ---
  if /I "%%E"=="NeuralNetwork" (
    REM NeuralNetwork needs deterministic RNG; use chacha8 with a fixed seed
    cargo run --bin tester -- "Examples\%%E" --rng=chacha8 --seed=42
  ) else (
    cargo run --bin tester -- "Examples\%%E"
  )
  if errorlevel 1 goto :error
)

echo.
echo Version 2 AST conversion
cargo build --bins
cargo run --bin ast-v2 -- Examples/HelloWorld
cargo run --bin tester -- "conversion\Examples\HelloWorld"
echo .

echo.
echo All examples converted and tested successfully.
exit /b 0

:error
echo.
echo Script failed with error %ERRORLEVEL%.
exit /b %ERRORLEVEL%
