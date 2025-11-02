// Converted from Rust: struct NeuralNetwork
interface NeuralNetwork<T> {
  x_layers: number;
  y_nodes: number;
  z_weights: number;
  data: T[];
}

export const NeuralNetwork: any = {};

// Converted from Rust: impl NeuralNetwork associated functions
Object.assign(NeuralNetwork, {
  new_with_value: function(x_layers: number, y_nodes: number, z_weights: number, value: any) {
    const size = x_layers * y_nodes * z_weights;
    const data = Array(size).fill(value);
    return {
      x_layers, y_nodes, z_weights, data,
      dims() { return [this.x_layers, this.y_nodes, this.z_weights]; },
      len() { return this.data.length; },
      is_empty() { return this.data.length === 0; },
      index(x: number, y: number, z: number) {
        if (x < this.x_layers && y < this.y_nodes && z < this.z_weights) {
          const yz = this.y_nodes * this.z_weights;
          return x * yz + y * this.z_weights + z;
        }
        return undefined;
      },
      get(x: number, y: number, z: number) {
        const i = this.index(x, y, z);
        return i === undefined ? undefined : this.data[i];
      },
      get_mut(x: number, y: number, z: number) {
        const i = this.index(x, y, z);
        return i === undefined ? undefined : this.data[i];
      },
    };
  },
});

// Converted from Rust: impl NeuralNetwork<T>::new_with_value
function NeuralNetwork_new_with_value<T>(x_layers: number, y_nodes: number, z_weights: number, value: T): any {
  // Rust variable declaration
  const size = x_layers.checked_mul(y_nodes).and_then(/* Unsupported expression: | v | v . checked_mul (z_weights) */).expect("dimensions too large (overflow)");
  // Unsupported trailing expression
}

// Converted from Rust: impl NeuralNetwork<T>::dims
function NeuralNetwork_dims<T>(selfObj: NeuralNetwork<T>): any {
  return [selfObj.x_layers, selfObj.y_nodes, selfObj.z_weights];
}

// Converted from Rust: impl NeuralNetwork<T>::len
function NeuralNetwork_len<T>(selfObj: NeuralNetwork<T>): number {
  return selfObj.data.len();
}

// Converted from Rust: impl NeuralNetwork<T>::is_empty
function NeuralNetwork_is_empty<T>(selfObj: NeuralNetwork<T>): boolean {
  return selfObj.data.is_empty();
}

// Converted from Rust: impl NeuralNetwork<T>::index
function NeuralNetwork_index<T>(selfObj: NeuralNetwork<T>, x: number, y: number, z: number): number | undefined {
  // Rust if
  if (x < selfObj.x_layers && y < selfObj.y_nodes && z < selfObj.z_weights) {
  // Rust variable declaration
  const yz = selfObj.y_nodes * selfObj.z_weights;
  return Some(x * yz + y * selfObj.z_weights + z);
  } else {
  return None;
  }
}

// Converted from Rust: impl NeuralNetwork<T>::get
function NeuralNetwork_get<T>(selfObj: NeuralNetwork<T>, x: number, y: number, z: number): T | undefined {
  return selfObj.index(x, y, z).map(/* Unsupported expression: | i | & self . data [i] */);
}

// Converted from Rust: impl NeuralNetwork<T>::get_mut
function NeuralNetwork_get_mut<T>(selfObj: NeuralNetwork<T>, x: number, y: number, z: number): T | undefined {
  return selfObj.index(x, y, z).map(/* Unsupported expression: move | i | & mut self . data [i] */);
}

// Converted from Rust: impl NeuralNetwork<T>::for_each
function NeuralNetwork_for_each<T, F>(selfObj: NeuralNetwork<T>, f: F): void {
  // Rust variable declaration
  const [x_max, y_max, z_max] = selfObj.dims() as any;
  // Rust for-loop
  for (let x = 0; x < x_max; x++) {
  // Rust for-loop
  for (let y = 0; y < y_max; y++) {
  // Rust for-loop
  for (let z = 0; z < z_max; z++) {
  // Rust variable declaration
  const idx = x * /* Unsupported expression: (y_max * z_max) */ + y * z_max + z;
  // Rust expression
  f(x, y, z, /* Unsupported expression: & self . data [idx] */);
  }
  }
  }
}

// Converted from Rust: impl NeuralNetwork<T>::for_each_mut
function NeuralNetwork_for_each_mut<T, F>(selfObj: NeuralNetwork<T>, f: F): void {
  // Rust variable declaration
  const [x_max, y_max, z_max] = selfObj.dims() as any;
  // Rust for-loop
  for (let x = 0; x < x_max; x++) {
  // Rust for-loop
  for (let y = 0; y < y_max; y++) {
  // Rust for-loop
  for (let z = 0; z < z_max; z++) {
  // Rust variable declaration
  const idx = x * /* Unsupported expression: (y_max * z_max) */ + y * z_max + z;
  // Rust expression
  f(x, y, z, /* Unsupported expression: & mut self . data [idx] */);
  }
  }
  }
}

// Converted from Rust: impl NeuralNetwork associated functions
Object.assign(NeuralNetwork, {
  random_uniform: function(x_layers: number, y_nodes: number, z_weights: number, low: number, high: number) {
    function mulberry32(a:number){return function(){let t=a+=0x6D2B79F5;t=Math.imul(t^t>>>15,t|1);t^=t+Math.imul(t^t>>>7,t|61);return ((t^t>>>14)>>>0)/4294967296;}}
    const seed = (globalThis as any).__RUST_TO_TS_SEED >>> 0 || 0xDEADBEEF;
    const rand = mulberry32(seed);
    const size = x_layers * y_nodes * z_weights;
    const data: number[] = new Array(size);
    for (let i = 0; i < size; i++) { const r = rand(); data[i] = low + r * (high - low); }
    return {
      x_layers, y_nodes, z_weights, data,
      dims() { return [this.x_layers, this.y_nodes, this.z_weights]; },
      len() { return this.data.length; },
      is_empty() { return this.data.length === 0; },
      index(x: number, y: number, z: number) {
        if (x < this.x_layers && y < this.y_nodes && z < this.z_weights) {
          const yz = this.y_nodes * this.z_weights;
          return x * yz + y * this.z_weights + z;
        }
        return undefined;
      },
      get(x: number, y: number, z: number) {
        const i = this.index(x, y, z);
        return i === undefined ? undefined : this.data[i];
      },
      get_mut(x: number, y: number, z: number) {
        const i = this.index(x, y, z);
        return i === undefined ? undefined : this.data[i];
      },
    };
  },
});

// Converted from Rust: impl NeuralNetwork<T>::random_uniform
function NeuralNetwork_random_uniform<T>(x_layers: number, y_nodes: number, z_weights: number, low: T, high: T): any {
  // Rust variable declaration
  const size = x_layers.checked_mul(y_nodes).and_then(/* Unsupported expression: | v | v . checked_mul (z_weights) */).expect("dimensions too large (overflow)");
  // Rust variable declaration
  let rng = rand.thread_rng();
  // Rust variable declaration
  const dist = Uniform.new(low, high);
  // Rust variable declaration
  let data = Vec.with_capacity(size);
  // Unsupported for-loop pattern: for _ in 0 .. size { data . push (dist . sample (& mut rng)) ; } . pat
  // Original: for _ in 0 .. size { data . push (dist . sample (& mut rng)) ; }
  // Unsupported trailing expression
}

