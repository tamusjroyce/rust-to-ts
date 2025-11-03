# Rust ⇄ TypeScript Converter & Tester

##Work in progress

The purpose of this code is a Proof of Concept that will later be turned into a MCP. So code conversions are first done by this tool. Then for the parts it can't convert. It uses AI to try and update this conversion tool. The translator continuously improves itself. And if that doesnt work. It hand it off to AI to try and make the conversion. A patch tracking those changes along the way

The complex part is library conversions. This tends to add library specific hard-coding

Another part is to build two-way binding between both languages

A third is a GUI debuggable scratch or node-red/n8n like workflow as a language to dual bind with

A forth is the neural network example, 3D graphing it in a UI across languages

A fifth is using interpolation to build a similar 3d graph

A sixth is using non-linear regression + kspace to combine two trained ANN'a into a similar one that training a regulae ANN would produce

This workspace contains:

- A Rust→TypeScript converter binary (`rust-to-ts`) that walks a folder (e.g. `Examples/*`), converts `.rs` files to `.ts` side-by-side, and keeps unsupported Rust embedded as comments.
- A cross-runtime tester binary (`tester`) that runs a Rust example (via Cargo) and the converted TypeScript (via Deno), then compares the console output.

It’s designed to help iterate on runnable parity between Rust sample code and its generated TypeScript counterpart.

Note: This project and the docs were largely “vibed” using Copilot Pro (GPT-5) to accelerate iteration and polish.

## How it works

- Converter
  - Parses Rust using `syn/quote/proc-macro2` and emits TypeScript.
  - Adds simple mappings (println! → console.log, for-ranges, if-let Some, Option/Vec types, etc.).
  - For example `main.rs`, injects imports for sibling `lib.ts` and exports `main()`.
  - For `Examples/NeuralNetwork`, it emits a TS runtime object for `NeuralNetwork` plus helpers to keep it runnable.
  - Unsupported constructs are preserved as comments so the TS still parses and runs.

- Tester
  - Detects Cargo examples and runs them with `cargo run`.
  - Executes TypeScript with Deno via a tiny wrapper that imports and calls `main()`.
  - For seeded randomness parity, the wrapper injects globals so TS can pick the same RNG algorithm and seed.
  - Prints both outputs with labels and checks exact equality.

## Requirements

- Rust toolchain (Cargo)
- Deno (to run the generated TypeScript)

## Build

```powershell
cargo build --bins
```

## Convert an example

```powershell
cargo run --bin rust-to-ts -- Examples/NeuralNetwork
```

This will write `src/*.ts` next to each `src/*.rs` inside `Examples/NeuralNetwork`.

## Run tester (exact parity example)

NeuralNetwork supports seeded RNG selection that is mirrored in TS to achieve exact parity. For example, ChaCha8 with a fixed seed:

```powershell
cargo run --bin tester -- Examples/NeuralNetwork --rng=chacha8 --seed=42
```

Sample output:

```
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
Running `target\debug\tester.exe Examples/NeuralNetwork --rng=chacha8 --seed=42`
Outputs match exactly.
--- Rust ---
RNG: chacha8
NeuralNetwork<f64> dims: x(layers)=3, y(nodes)=4, z(weights)=2 | total elements=24
nn[0, 0, 0] = -0.260221678763628
nn[0, 0, 1] = 0.8884351472370327
nn[0, 1, 0] = 0.12749235378578305
nn[0, 1, 1] = -0.8611383140087128
nn[1, 0, 0] = 0.2225218079984188
nn[1, 0, 1] = 0.017620211001485586
nn[1, 1, 0] = 0.39068748988211155
nn[1, 1, 1] = 0.44959122501313686

--- Deno ---
RNG: chacha8
NeuralNetwork<f64> dims: x(layers)=3, y(nodes)=4, z(weights)=2 | total elements=24
nn[0, 0, 0] = -0.260221678763628
nn[0, 0, 1] = 0.8884351472370327
nn[0, 1, 0] = 0.12749235378578305
nn[0, 1, 1] = -0.8611383140087128
nn[1, 0, 0] = 0.2225218079984188
nn[1, 0, 1] = 0.017620211001485586
nn[1, 1, 0] = 0.39068748988211155
nn[1, 1, 1] = 0.44959122501313686
```

Other RNGs supported (Rust and TS): `mulberry64`, `chacha8`. PCG64 support on TS can be added similarly if needed.

## Tips

- If TS fails to parse due to unsupported Rust constructs, the converter emits placeholders so the file still runs; unsupported parts are annotated as comments for future improvements.
- The converter writes files side-by-side; you can re-run it safely as you iterate on Rust sources.

## Credits

- Copyright © 2025 TamusJRoyce. Licensed under MIT.
- This project and documentation were largely “vibed” using Copilot Pro (GPT-5).

## License

MIT — see `LICENSE`.

See `THIRD_PARTY_NOTICES.md` for dependency licenses and algorithm provenance notes. No GPL/LGPL/AGPL code is included.

## Disclaimer (copilot driven):

- Project license
  - Top-level `Cargo.toml` declares `license = "MIT"`.
  - `LICENSE` is present with MIT terms.
  - `README.md` aligns with MIT.

- Rust dependencies and their licenses
  - Converter/Tester:
    - syn (MIT OR Apache-2.0)
    - quote (MIT OR Apache-2.0)
    - proc-macro2 (MIT OR Apache-2.0)
  - Example `Examples/NeuralNetwork`:
    - rand (MIT OR Apache-2.0), which may pull transitive crates like `rand_core` and `rand_chacha` (also MIT OR Apache-2.0)

- Generated/embedded TypeScript runtime bits
  - mulberry32 helper: well-known CC0/public-domain snippet; acknowledged in `THIRD_PARTY_NOTICES.md`.
  - splitmix64 (Mulberry64-like step): algorithmic constants; implementation here is original (no third-party code copied).
  - ChaCha8 (reduced-round): algorithm by D. J. Bernstein; TypeScript and Rust implementations here are original.
  - PCG64 note: the Rust "PCG64-like" is a minimal, original demo; TS currently falls back. If the official PCG reference code is used in the future, it’s under Apache-2.0 and will be noted accordingly.

- Codebase scan
  - No files contain GPL/LGPL/AGPL headers or references.
  - No vendored third-party source files.
  - Only permissive-license crates are used.

- Conclusion
  - No GPL/LGPL/AGPL code detected in this repository.
  - All dependencies are permissively licensed and compatible with MIT.
  - Runtime snippets are original or public domain.
