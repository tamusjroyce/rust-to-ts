Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# Build all binaries
cargo build --bins

# Run ast-v2 conversions into conversion/
cargo run --bin ast-v2 -- Examples/HelloWorld
cargo run --bin ast-v2 -- Examples/NeuralNetwork
cargo run --bin ast-v2 -- --both Examples/src/converter

# Run tester parity checks on converted examples
cargo run --bin tester -- conversion/Examples/HelloWorld
cargo run --bin tester -- conversion/Examples/NeuralNetwork -- --rng=chacha8 --seed=42
cargo run --bin tester -- conversion/Examples/src/converter
