# Project Plan: AST v2 & Example Coverage

## High‑Level Goal
Make `ast_v2` the primary, AST‑driven Rust ⇄ TypeScript converter for all examples (including the NeuralNetwork example), and validate parity using the existing `tester` binary.

## Requirements (from owner)
- "I would like ast_v2 to be able to convert rust to typescript for all of the examples including the neural network example.
  - and to validate it using src/Tester to output the results of rust and of typescript and compare them for differences. They should be the same.
  - ast_v2 should use the custom ast data structure, unlike the previous ast which did rule based conversions.
  - ast_v2 should only depend on the libraries it is currently depending on.
  - I would also like ast_v2 to convert that same typescript back to javascript.
  - By default, ast_v2 should be creating a folder it places the .ts file from parent folder .ts file. That way converting from ts -> .rs doesn't overwrite the original rust file."

## Concrete Objectives
- Extend `ast_v2` so it can:
  - Parse and convert all Rust examples under `Examples/` (HelloWorld, NeuralNetwork, `Examples/src`) into TypeScript using the shared AST (`Module`, `TypeDecl`, `Function`, `TypeRef`, etc.).
  - Emit TS for NeuralNetwork that matches the existing hand‑written implementation closely enough that `src/Tester` reports identical output for Rust vs Deno.
  - Convert the generated TypeScript back into Rust without overwriting the original `.rs` sources.
  - Eventually support **true bidirectional parsing and conversion on both sides**: the Rust implementation already parses Rust and emits TS via the shared AST; the TypeScript side is expected to gain a _real_ Rust parser and converter (or an equivalent bridge) so that TS can independently drive Rust→AST→TS and TS→AST→Rust as well, not just host stubs.

## Scope & Non‑Goals (for now)
- Supported surface area is intentionally **minimal and example‑driven**:
  - Only Rust and TypeScript features actually exercised in the three example trees are in scope:
    - `Examples/HelloWorld`
    - `Examples/NeuralNetwork`
    - `Examples/src` (the project‑self example mirror)
  - If a Rust/TS feature does not appear in those examples, `ast_v2` does **not** need to support converting it at this time.
- Out‑of‑scope features (anything beyond the examples) are expected to be handled later by a GPT‑5.1 based agent wired in via an MCP architecture, which can:
  - Detect constructs the minimal converter cannot handle.
  - Perform best‑effort or manual‑review conversions for those unsupported cases.
- Keep `ast_v2` dependencies limited to what is currently declared in `Cargo.toml` (no new crates).

### Strict "No Wrapper" Rule
- Conversions must always be implemented **natively** in the language that is running them (Rust in the Rust crate, TypeScript in the TS side) using the shared AST model.
- `ast_v2` (or any future TS companion) must **never** shell out to an external converter binary, call back into the Rust `ast-v2` process, or use any subprocess/IPC "wrapper" to perform conversions.
- When the TypeScript side gains Rust→AST→TS or TS→AST→Rust capabilities, they must be implemented as real, native logic (or via a first‑class library binding), not by delegating work to the Rust binary through a wrapper.

## Conversion & Layout Rules
- `ast_v2` must:
  - Use its **custom AST layer** in `src/ast_v2/mod.rs` as the only source of truth for Rust⇄TS structure; avoid ad‑hoc string/regex rewriting.
  - Treat the `Module` / `TypeDecl` / `Function` / `TypeRef` graph as the **single, pure AST representation** used for all language conversions (Rust→AST→TS / TS→AST→Rust); new features must extend this AST model rather than bypassing it.
  - When converting `.rs → .ts` or `.ts → .rs`, write outputs under a designated output root (default: `conversion/`) that preserves the original directory structure.
  - Never overwrite original Rust example files under `Examples/`; converted TS/RS lives under `conversion/` by default.

## Validation Workflow
- For each example directory in `Examples/`:
  - Run `cargo run --bin ast-v2 -- Examples/<ExampleName> --output-dir conversion` to generate TS (and/or back‑converted RS).
  - Use `cargo run --bin tester -- conversion/Examples/<ExampleName> ...` or an equivalent flow to compare Rust vs Deno outputs on the converted artifacts.
  - Outputs from Rust and Deno must match exactly; any divergence is treated as a regression.

## BPMN Editor (Tauri) — UX + File Workflow

Goal: make the desktop BPMN editor reliably support `New`, `Open`, `Save`, `Save As`, `Convert`, and `Validate`, with correct prompts and persistence.

### Tracking Rules
- Use task checkboxes below.
- A task is only marked `[x]` after its **Validation** steps are run and confirmed.

### Tasks

- [ ] **New**: detect unsaved changes vs default template
  - Requirements:
    - If editor contents differ from the default BPMN template and/or there are unsaved edits, prompt: **Save & New**, **Discard**, **Cancel**.
    - If unchanged (matches default template / no unsaved edits), no prompt; load default template into the BPMN XML editor and canvas.
  - Validation:
    - Modify BPMN XML, click New ⇒ prompt appears; each option behaves as expected.
    - With a fresh default template, click New ⇒ no prompt and template stays.

- [ ] **Open**: file dialog + persistent last folder
  - Requirements:
    - Use a native file open dialog to pick a `.bpmn` file.
    - Remember the last folder used for Open; persist across app restart.
  - Validation:
    - Open a file in folder A, restart app, Open again ⇒ dialog starts in folder A.

- [ ] **Save**: disabled until a file is opened; external-modification detection
  - Requirements:
    - Save button is disabled/greyed out until a file has been opened (or a path is chosen via Save As).
    - If file on disk changed since it was opened (or since last Save), prompt: **Overwrite** or **Cancel**.
    - Otherwise save silently.
  - Validation:
    - Open file, edit outside app, then Save ⇒ overwrite prompt appears.
    - With no external change, Save writes without prompting.

- [ ] **Save As**: choose path; confirm overwrite
  - Requirements:
    - Save As opens a native save dialog.
    - If target exists, prompt overwrite/cancel.
    - After Save As, the chosen path becomes the current file (enables Save).
  - Validation:
    - Save As to new path creates file.
    - Save As to existing path prompts.

- [ ] **Convert**: save-first with Save rules, then run conversions
  - Requirements:
    - Convert triggers the same save behavior as Save (including external-change prompt).
    - After saving, run:
      - BPMN → Rust
      - BPMN → Rust → JavaScript
    - Output should be visible in the UI (Rust tab / TS tab / JS tab or explicit output panel).
  - Validation:
    - With unsaved edits, Convert prompts to save; after save, outputs update.

- [ ] **Validate**: run validator and show console outputs
  - Requirements:
    - Runs round-trip validation and shows the Rust/Deno outputs in a window/panel.
    - Status should clearly indicate PASS/FAIL.
  - Validation:
    - Validate on known-good example shows PASS and outputs.
    - Intentionally break a directive and Validate shows FAIL with details.

## Implementation Notes for Future Work
- Start from the existing HelloWorld mappings in `src/ast_v2/mod.rs` (`from_rust_module`, `from_ts_module`) and generalize them to handle:
  - Structs, functions, and basic types used across `Examples/src`.
  - RNG abstraction and NeuralNetwork data structures used in `Examples/NeuralNetwork/src/lib.rs` and `main.rs`.
- When adding features, extend the shared AST model first, then implement both Rust→AST and TS→AST, and finally AST→Rust/TS emitters.
- Keep generated TS idempotent where possible: re‑running `ast_v2` over the same Rust sources should not introduce noisy diffs in `conversion/`.

## Planned ast_v2 Refactor (File Layout)
- Consolidate the **shared AST data structures** into a single module (e.g. `src/ast_v2/ast.rs`):
  - Defines `Module`, `TypeDecl`, `Field`, `Param`, `Function`, `TypeKind`, `TypeRef`, `FunctionKind` only.
  - Contains **no** Rust- or TS-specific parsing or printing logic.
- Create a **Rust-specific AST adapter** (e.g. `src/ast_v2/rust_ast.rs`):
  - Functions to map from Rust/syn → shared AST, and from shared AST → Rust code strings.
  - Depends on `syn`/`quote` but not on any TS APIs.
- Create a **TypeScript-specific AST adapter** (e.g. `src/ast_v2/ts_ast.rs`):
  - Functions to map from TS source text → shared AST, and from shared AST → TS code strings.
  - Pure string/TS-syntax handling, no Rust/syn dependencies.
- Add a **Rust→TS converter module** (e.g. `src/ast_v2/convert_rust_to_ts.rs`):
  - Orchestrates Rust→shared AST→TS using the two language adapters.
  - Exposes entrypoints used by the `ast-v2` binary for `.rs → .ts`.
- Add a **TS→Rust converter module** (e.g. `src/ast_v2/convert_ts_to_rust.rs`):
  - Orchestrates TS→shared AST→Rust using the same shared AST model.
  - Exposes entrypoints used by the `ast-v2` binary for `.ts → .rs`.
- Keep `src/ast_v2/mod.rs` as a thin façade that re-exports the shared AST and these directional converters without embedding conversion logic itself.