// Support library for converted TypeScript examples
// Minimal stubs to let generated code run.

export function make_rng_from_args(args?: any): any {
  return { next: () => Math.random() };
}

export class NeuralNetwork<T> {
  constructor(public layers: number = 0) {}
}

export const env = {
  args() {
    // crude simulation; real implementation would parse process.argv in Deno/node
    const a = (globalThis as any).__RUST_TO_TS_ARGS || [];
    return {
      skip(n: number) { return { collect: () => a.slice(n) }; },
      collect: () => a.slice(),
    } as any;
  }
};

export const std = {
  process: {
    exit(code: number) { /* no-op for now */ },
  },
  env,
};
