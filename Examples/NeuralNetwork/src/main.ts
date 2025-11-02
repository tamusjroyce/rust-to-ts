import { NeuralNetwork } from "./lib.ts";

// Converted from Rust: fn main(...)
export function main(): void {
  // Rust variable declaration
  const x_layers = 3;
  // Rust variable declaration
  const y_nodes = 4;
  // Rust variable declaration
  const z_weights = 2;
  // Rust variable declaration
  const nn = NeuralNetwork.random_uniform(x_layers, y_nodes, z_weights, -1.0, 1.0);
  // Rust variable declaration
  const [x, y, z] = nn.dims() as any;
  // Rust macro
  console.log(`NeuralNetwork<f32> dims: x(layers)=${x}, y(nodes)=${y}, z(weights)=${z} | total elements=${nn.len()}`);
  // Rust for-loop
  for (let layer = 0; layer < Math.min(x, 2); layer++) {
  // Rust for-loop
  for (let node = 0; node < Math.min(y, 2); node++) {
  // Rust for-loop
  for (let weight = 0; weight < Math.min(z, 2); weight++) {
  // Rust if-let
  const __tmp = nn.get(layer, node, weight);
  if (__tmp !== undefined) {
    const val = __tmp;
  // Rust macro
  console.log(`nn[${layer}, ${node}, ${weight}] = ${val}`);
  }
  }
  }
  }
}

