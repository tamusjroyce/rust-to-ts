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
    const argv = (globalThis as any).__RUST_TO_TS_ARGS || [];
    return {
      skip(n: number) { return { collect: () => argv.slice(n) }; },
      collect: () => argv.slice(),
    } as any;
  }
};

// expose global for generated code using bare `env`/`std`
(globalThis as any).env = env;

export const std = {
  process: {
    exit(code: number) { /* no-op stub */ },
  },
  env,
};
(globalThis as any).std = std;
