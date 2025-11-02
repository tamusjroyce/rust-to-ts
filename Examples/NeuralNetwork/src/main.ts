import { NeuralNetwork } from "./lib.ts";

// Converted from Rust: fn main(...)
export function main(): void {
  const x_layers = 3;
  const y_nodes = 4;
  const z_weights = 2;

  // Initialize with random values in [-1.0, 1.0]
  const nn: NeuralNetwork<number> = NeuralNetwork.random_uniform(x_layers, y_nodes, z_weights, -1.0, 1.0);

  const [x, y, z] = nn.dims();
  console.log(`NeuralNetwork<f32> dims: x(layers)=${x}, y(nodes)=${y}, z(weights)=${z} | total elements=${nn.len()}`);

  // Print a small sample to verify (match Rust's bounds and order)
  for (let layer = 0; layer < Math.min(x, 2); layer++) {
    for (let node = 0; node < Math.min(y, 2); node++) {
      for (let weight = 0; weight < Math.min(z, 2); weight++) {
        const val = nn.get(layer, node, weight);
        if (val !== undefined) {
          console.log(`nn[${layer}, ${node}, ${weight}] = ${val}`);
        }
      }
    }
  }
}

