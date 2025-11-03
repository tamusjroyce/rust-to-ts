import { NeuralNetwork, make_rng_from_args } from "./lib.ts";

// Converted from Rust: fn main(...)
export function main(): void {
  // Rust variable declaration
  const x_layers = 3;
  // Rust variable declaration
  const y_nodes = 4;
  // Rust variable declaration
  const z_weights = 2;
  // Rust variable declaration
  const rng_label = (((globalThis as any).__RUST_TO_TS_RNG||'default') as string);
  // Rust variable declaration
  let rng = ({ next_f32: (low: number, high: number) => { function mulberry32(a:number){return function(){let t=a+=0x6D2B79F5;t=Math.imul(t^t>>>15,t|1);t^=t+Math.imul(t^t>>>7,t|61);return ((t^t>>>14)>>>0)/4294967296;}} const seed=(globalThis as any).__RUST_TO_TS_SEED>>>0||0xDEADBEEF; const rand=mulberry32(seed); return low + rand() * (high - low); } });
  // Rust variable declaration
  const nn = NeuralNetwork.random_uniform(x_layers, y_nodes, z_weights, -1.0, 1.0);
  // Rust macro
  console.log(`RNG: ${rng_label}`);
  // Rust variable declaration
  const [x, y, z] = nn.dims() as any;
  // Rust macro
  console.log(`NeuralNetwork<f64> dims: x(layers)=${x}, y(nodes)=${y}, z(weights)=${z} | total elements=${nn.len()}`);
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

