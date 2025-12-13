#!/usr/bin/env bash
set -euo pipefail

# Change to repo root (folder containing this script)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Building Rust binaries (rust-to-ts, tester) ==="
cargo build --bins

echo
EXAMPLES=("HelloWorld" "NeuralNetwork" "src")

echo "=== Converting Rust -> TypeScript for all examples ==="
for ex in "${EXAMPLES[@]}"; do
  echo
  echo "--- Converting Examples/$ex ---"
  cargo run --bin rust-to-ts -- "Examples/$ex"
done

echo
echo "=== Running tester for all examples ==="
for ex in "${EXAMPLES[@]}"; do
  echo
  echo "--- Testing Examples/$ex ---"
  if [[ "$ex" == "NeuralNetwork" ]]; then
    # NeuralNetwork needs deterministic RNG; use chacha8 with a fixed seed
    cargo run --bin tester -- "Examples/$ex" --rng=chacha8 --seed=42
  else
    cargo run --bin tester -- "Examples/$ex"
  fi
done

echo
echo "All examples converted and tested successfully."
