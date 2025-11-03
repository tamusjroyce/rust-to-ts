Third‑Party Notices

This project is licensed under MIT (see LICENSE). It also uses or references the following third‑party components and algorithms. No GPL/LGPL/AGPL code is included.

- Rust crates used by the converter and tester (build-time/runtime):
  - syn (MIT OR Apache-2.0)
  - quote (MIT OR Apache-2.0)
  - proc-macro2 (MIT OR Apache-2.0)
  - rand (MIT OR Apache-2.0) in the NeuralNetwork example; this may pull transitive crates like rand_chacha and rand_core, which are also MIT OR Apache-2.0.

- Algorithms implemented in original code:
  - SplitMix64 (aka Mulberry64-style step): algorithm by Steele/Vigna; commonly released into the public domain/CC0 by authors in reference material. This repository contains an original implementation; no third‑party code was copied.
  - ChaCha (reduced-round ChaCha8 used as a PRNG): algorithm by D. J. Bernstein; algorithm descriptions are not copyrightable. The TypeScript and Rust implementations here are original, written from the specification; no third‑party code was copied.
  - PCG family: algorithm by Melissa O’Neill. This repository includes a minimal, original "PCG64-like" Rust implementation for demonstration; it is not copied from the PCG reference code. If the official PCG reference implementation is used in the future, it is available under the Apache-2.0 license and appropriate notices should be added.

- Small JavaScript PRNG helper:
  - mulberry32 one-liner used in TypeScript runtime scaffolding is a well-known snippet published as public domain/CC0 in community Q&A resources. It is included inline; attribution is not required but acknowledged here for clarity.

Summary

- No GPL/LGPL/AGPL code has been introduced by this repository.
- All dependencies are under permissive licenses (MIT/Apache-2.0) and compatible with this project’s MIT license.
- Implementations of algorithms are original and do not incorporate third‑party source code.
