import { NeuralNetwork } from "./lib.ts";

// Converted from Rust: fn main(...) for NeuralNetwork example
export function main(): void {
  const x_layers = 3;
  const y_nodes = 4;
  const z_weights = 2;
  const rng_label = (((globalThis as any).__RUST_TO_TS_RNG || 'default') as string);
  let rng = ({ next_f32: (low: number, high: number) => {
    function mulberry32(a: number) {
      return function() {
        let t = a += 0x6D2B79F5;
        t = Math.imul(t ^ (t >>> 15), t | 1);
        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
      };
    }
    const seed = ((globalThis as any).__RUST_TO_TS_SEED >>> 0) || 0xDEADBEEF;
    const rand = mulberry32(seed);
    return low + rand() * (high - low);
  }});
  const nn = (NeuralNetwork as any).random_uniform(x_layers, y_nodes, z_weights, -1.0, 1.0);
  console.log(`RNG: ${rng_label}`);
  const [x, y, z] = nn.dims() as any;
  console.log(`NeuralNetwork<f64> dims: x(layers)=${x}, y(nodes)=${y}, z(weights)=${z} | total elements=${nn.len()}`);
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
