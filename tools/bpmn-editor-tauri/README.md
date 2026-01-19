# BPMN Editor (Tauri)

This is a lightweight BPMN editor UI built with `bpmn-js`, packaged as a native desktop app via Tauri.

## Prereqs (Windows)
- Rust toolchain (you already have `cargo`).
- Node.js (required for Tauri frontend tooling + `@tauri-apps/cli`).
- WebView2 runtime (usually already installed on Windows 10/11).

## Run (dev)
From this folder:
- `npm install`
- `npm run dev`

Tauri will run the Vite dev server and open the desktop window.

## Notes
- This is currently a UI-only editor (open via file picker + save via download).
- Next step: add Tauri commands to integrate with the repo’s BPMN↔Rust conversion + validator.
