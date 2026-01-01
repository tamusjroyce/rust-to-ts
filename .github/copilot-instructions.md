# Copilot Instructions for rust-to-ts

## Big Picture
- This repo is a **Rust ⇄ TypeScript converter + cross-runtime tester** for runnable examples.
- Main goals: (1) convert Rust examples to TS while preserving behavior, (2) compare Rust vs TS output via Deno, (3) evolve an AST-based, bidirectional converter (AST v2).

## Key Binaries & Flows
- `rust-to-ts` (classic converter): [src/converter/main.rs](src/converter/main.rs)
  - Walks given roots (e.g. `Examples/HelloWorld`, `Examples/NeuralNetwork`, `Examples/src`).
  - Uses `converter::convert_rs_dir_to_ts_side_by_side` to emit `.ts` files **next to** `.rs` files.
  - Unsupported Rust is preserved as comments so TS still parses and runs.
- `tester` (Rust vs Deno parity): [src/Tester/main.rs](src/Tester/main.rs)
  - Discovers matching `<stem>.rs` / `<stem>.ts` under a target directory (prefers `main.rs`/`main.ts`).
  - Runs Rust via `cargo run` if a `Cargo.toml` exists, otherwise via `rustc` into `target/tmp/`.
  - Runs TS via Deno using an auto-generated wrapper in `target/tmp/*_deno_run.ts` that imports the TS module and calls `main()`.
  - Compares stdout exactly and prints labeled sections (`--- Rust ---`, `--- Deno ---`).
- `ast-v2` (bidirectional AST experiment): [src/ast_v2/main.rs](src/ast_v2/main.rs)
  - CLI: `cargo run --bin ast-v2 -- [--dry-run] [--output-dir DIR] <path...>`.
  - Uses [src/ast_v2/mod.rs](src/ast_v2/mod.rs) to parse Rust/TS into a shared `Module` / `TypeDecl` / `Function` model and convert `*.rs ⇄ *.ts`.
  - Writes results under `conversion/` (or a custom `--output-dir`), preserving the original directory layout.

## Project Layout to Know
- Classic converter pipeline:
  - AST layer: [src/ast/mod.rs](src/ast/mod.rs), [src/ast/ast_syn.rs](src/ast/ast_syn.rs).
  - Converter logic: [src/converter/converter.rs](src/converter/converter.rs).
  - TS runtime shims used by generated code: [src/converter/lib.ts](src/converter/lib.ts).
- AST v2 pipeline:
  - Core AST+conversion logic: [src/ast_v2/mod.rs](src/ast_v2/mod.rs).
  - Dry-run harness (not wired as a Cargo bin by default): [src/ast_v2_dry_run/main.rs](src/ast_v2_dry_run/main.rs).
- Examples:
  - Simple struct/function mapping: [Examples/HelloWorld/hello_world.rs](Examples/HelloWorld/hello_world.rs) and its TS.
  - RNG + neural net parity: [Examples/NeuralNetwork/src/lib.rs](Examples/NeuralNetwork/src/lib.rs) & [Examples/NeuralNetwork/src/main.rs](Examples/NeuralNetwork/src/main.rs) with TS counterparts.
  - `Examples/src` mirrors the root `src/` to test converting the project’s own bins.

## Core Workflows (run from repo root)
- Build all bins: `cargo build --bins`.
- Convert examples with classic pipeline:
  - `cargo run --bin rust-to-ts -- Examples/HelloWorld`
  - `cargo run --bin rust-to-ts -- Examples/NeuralNetwork`
  - `cargo run --bin rust-to-ts -- Examples/src`
- Run tester for parity checks:
  - Simple: `cargo run --bin tester -- Examples/HelloWorld`.
  - Seeded RNG parity (NeuralNetwork): `cargo run --bin tester -- Examples/NeuralNetwork --rng=chacha8 --seed=42`.
- Experiment with AST v2 round-trips:
  - One-shot: `cargo run --bin ast-v2 -- Examples/HelloWorld --output-dir conversion`.
  - Dry run (no writes): `cargo run --bin ast-v2 -- --dry-run Examples/HelloWorld`.

## Conventions & Patterns
- **Side-by-side Rust/TS**
  - Converters assume `.rs` and `.ts` with the same stem live side-by-side; tester also discovers pairs this way.
  - Examples should export a `main()` in TS; the Deno wrapper calls `mod.main()` or falls back to a global `main`.
- **Runtime shims & environment**
  - TS helper runtime lives in [src/converter/lib.ts](src/converter/lib.ts), exposing `env`, `std`, and RNG glue via `globalThis`.
  - `tester`’s Deno wrapper sets globals like `__RUST_TO_TS_ARGS`, `__RUST_TO_TS_SEED`, `__RUST_TO_TS_SEED_U64`, and `__RUST_TO_TS_RNG`; TS code should read from these instead of parsing `Deno.args` directly.
- **RNG parity (NeuralNetwork)**
  - Rust RNG abstraction and algorithms are defined in [Examples/NeuralNetwork/src/lib.rs](Examples/NeuralNetwork/src/lib.rs) (`DefaultRng`, `Mulberry64`, `Pcg64`, `ChaCha8Rng`).
  - TS must mirror these algorithms and CLI flags (`--rng=<name>`, `--seed=<u64>`) so `tester` sees bit-identical outputs.
  - When adding a new RNG, update **both** Rust and TS libs and ensure `tester`’s wrapper still passes the right globals.
- **AST v2 modeling**
  - Type mapping uses `TypeRef` (`Number`, `String`, `Bool`, `Custom(String)`) for both Rust (`syn::Type`) and TS (string-based type names).
  - `from_rust_hello_world` / `from_ts_hello_world` in [src/ast_v2/mod.rs](src/ast_v2/mod.rs) are the reference for how HelloWorld-style structs and functions should map.
  - New heuristics or language features for the v2 pipeline should go through this AST layer, not ad-hoc string rewriting.

## Gotchas for AI Agents
- Tools rely heavily on **relative paths from repo root**; avoid changing CWD assumptions in new code.
- Prefer returning `Result<_, String>` with friendly messages (as in `ast_v2::convert_*` and the bin `main` functions) over panicking.
- Keep generated `.ts` files **idempotent**: re-running converters over the same inputs should not introduce noisy diffs.
- When in doubt, validate behavior by:
  - Converting a focused example (`HelloWorld` or `NeuralNetwork`).
  - Running `tester` to confirm Rust vs Deno output remains identical.