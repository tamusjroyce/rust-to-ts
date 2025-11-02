// Minimal TypeScript implementation of the NeuralNetwork used by main.rs
export class NeuralNetwork<T> {
  constructor(
    public x_layers: number,
    public y_nodes: number,
    public z_weights: number,
    public data: T[],
  ) {}

  static new_with_value<T>(x_layers: number, y_nodes: number, z_weights: number, value: T): NeuralNetwork<T> {
    const size = x_layers * y_nodes * z_weights;
    return new NeuralNetwork(x_layers, y_nodes, z_weights, Array(size).fill(value));
  }

  dims(): [number, number, number] {
    return [this.x_layers, this.y_nodes, this.z_weights];
  }

  len(): number {
    return this.data.length;
  }

  is_empty(): boolean {
    return this.data.length === 0;
  }

  private index(x: number, y: number, z: number): number | undefined {
    if (x < this.x_layers && y < this.y_nodes && z < this.z_weights) {
      const yz = this.y_nodes * this.z_weights;
      return x * yz + y * this.z_weights + z;
    }
    return undefined;
  }

  get(x: number, y: number, z: number): T | undefined {
    const i = this.index(x, y, z);
    return i === undefined ? undefined : this.data[i];
  }

  get_mut(x: number, y: number, z: number): T | undefined {
    // TS lacks borrow semantics; expose same as get
    const i = this.index(x, y, z);
    return i === undefined ? undefined : this.data[i];
  }

  static random_uniform(x_layers: number, y_nodes: number, z_weights: number, low: number, high: number): NeuralNetwork<number> {
    const size = x_layers * y_nodes * z_weights;
    const data = new Array<number>(size);
    for (let i = 0; i < size; i++) {
      // Non-deterministic; tester will show mismatch vs Rust due to RNG
      const r = Math.random();
      data[i] = low + r * (high - low);
    }
    return new NeuralNetwork(x_layers, y_nodes, z_weights, data);
  }
}

