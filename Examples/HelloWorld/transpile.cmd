@echo off
cd /d "%~dp0\..\.."
rem Run the rust-to-ts binary from the repository root to generate the .ts file
cargo run -- "Examples\HelloWorld\hello_world.rs"
pause
