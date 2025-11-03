// Converted from Rust: struct NeuralNetwork
interface NeuralNetwork<T> {
  x_layers: number;
  y_nodes: number;
  z_weights: number;
  data: T[];
}

export const NeuralNetwork: any = {};

// Unsupported Rust item: pub trait RngSource { fn next_f32 (& mut self , low : f32 , high : f32) -> f32 ; fn next_f64 (& mut self , low : f64 , high : f64) -> f64 ; }

// Converted from Rust: struct DefaultRng
interface DefaultRng {
}

// Skipped trait implementation (not yet supported)

// Skipped trait implementation (not yet supported)

// Converted from Rust: struct Mulberry64
interface Mulberry64 {
  state: number;
}

// Converted from Rust: impl Mulberry64::new
function Mulberry64_new(seed: number): any {
  return (undefined as any) /* Unsupported expression: Self { state : seed } */;
}

// Skipped trait implementation (not yet supported)

// Converted from Rust: struct Pcg64
interface Pcg64 {
  state: u128;
  inc: u128;
}

// Converted from Rust: impl Pcg64::new
function Pcg64_new(seed: number): any {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  return (undefined as any) /* Unsupported expression: Self { state : s , inc } */;
}

// Converted from Rust: impl Pcg64::next_u64
function Pcg64_next_u64(selfObj: Pcg64): number {
  // Unsupported statement (unhandled variant)
  // Unsupported statement: self . state = self . state . wrapping_mul (MUL) . wrapping_add (self . inc)
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  return xorshifted.rotate_right(rot);
}

// Skipped trait implementation (not yet supported)

// Converted from Rust: struct ChaCha8Rng
interface ChaCha8Rng {
  state: any;
  idx: number;
  buf: any;
}

// Converted from Rust: impl ChaCha8Rng::new
function ChaCha8Rng_new(seed: number): any {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported statement: st [0] = 0x6170_7865
  // Unsupported statement: st [1] = 0x3320_646e
  // Unsupported statement: st [2] = 0x7962_2d32
  // Unsupported statement: st [3] = 0x6b20_6574
  // Rust variable declaration
  let sm = Mulberry64.new(seed);
  // Rust for-loop
  for (let i = 0; i < 8; i++) {
  // Unsupported statement: st [4 + i] = (sm . next_f32 (0.0 , f32 :: MAX) as u32) ^ (0x9E37_79B9u32 . wrapping_mul (i as u32 + 1))
  }
  // Unsupported statement: st [12] = 0
  // Unsupported statement: st [13] = 0
  // Unsupported statement: st [14] = (seed as u32) ^ 0xDEAD_BEEF
  // Unsupported statement: st [15] = ((seed >> 32) as u32) ^ 0xBADC_0FFE
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust expression
  rng.refill();
  return rng;
}

// Converted from Rust: impl ChaCha8Rng::quarter
function ChaCha8Rng_quarter(x: any, ai: number, bi: number, ci: number, di: number): void {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported statement: a = a . wrapping_add (b)
  // Rust expression
  d + a;
  // Unsupported statement: d = d . rotate_left (16)
  // Unsupported statement: c = c . wrapping_add (d)
  // Rust expression
  b + c;
  // Unsupported statement: b = b . rotate_left (12)
  // Unsupported statement: a = a . wrapping_add (b)
  // Rust expression
  d + a;
  // Unsupported statement: d = d . rotate_left (8)
  // Unsupported statement: c = c . wrapping_add (d)
  // Rust expression
  b + c;
  // Unsupported statement: b = b . rotate_left (7)
  // Unsupported statement: x [ai] = a
  // Unsupported statement: x [bi] = b
  // Unsupported statement: x [ci] = c
  // Unsupported statement: x [di] = d
}

// Converted from Rust: impl ChaCha8Rng::refill
function ChaCha8Rng_refill(selfObj: ChaCha8Rng): void {
  // Rust variable declaration
  let x = selfObj.state;
  // Unsupported for-loop pattern: for _ in 0 .. 8 { Self :: quarter (& mut x , 0 , 4 , 8 , 12) ; Self :: quarter (& mut x , 1 , 5 , 9 , 13) ; Self :: quarter (& mut x , 2 , 6 , 10 , 14) ; Self :: quarter (& mut x , 3 , 7 , 11 , 15) ; Self :: quarter (& mut x , 0 , 5 , 10 , 15) ; Self :: quarter (& mut x , 1 , 6 , 11 , 12) ; Self :: quarter (& mut x , 2 , 7 , 8 , 13) ; Self :: quarter (& mut x , 3 , 4 , 9 , 14) ; } . pat
  // Original: for _ in 0 .. 8 { Self :: quarter (& mut x , 0 , 4 , 8 , 12) ; Self :: quarter (& mut x , 1 , 5 , 9 , 13) ; Self :: quarter (& mut x , 2 , 6 , 10 , 14) ; Self :: quarter (& mut x , 3 , 7 , 11 , 15) ; Self :: quarter (& mut x , 0 , 5 , 10 , 15) ; Self :: quarter (& mut x , 1 , 6 , 11 , 12) ; Self :: quarter (& mut x , 2 , 7 , 8 , 13) ; Self :: quarter (& mut x , 3 , 4 , 9 , 14) ; }
  // Rust for-loop
  for (let i = 0; i < 16; i++) {
  // Unsupported statement: self . buf [i] = x [i] . wrapping_add (self . state [i])
  }
  // Unsupported statement: self . state [12] = self . state [12] . wrapping_add (1)
  // Rust if
  if ((undefined as any) /* Unsupported expression: self . state [12] */ === 0) {
  // Unsupported statement: self . state [13] = self . state [13] . wrapping_add (1)
  }
  // Unsupported statement: self . idx = 0
}

// Converted from Rust: impl ChaCha8Rng::next_u32
function ChaCha8Rng_next_u32(selfObj: ChaCha8Rng): number {
  // Rust if
  if (selfObj.idx >= 16) {
  // Rust expression
  selfObj.refill();
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust expression
  selfObj.idx + 1;
  return v;
}

// Skipped trait implementation (not yet supported)

// Unsupported Rust item: # [doc = " Algorithm selector, driven by CLI."] pub enum RngKind { Default , Mulberry64 (u64) , Pcg64 (u64) , ChaCha8 (u64) }

// Skipped trait implementation (not yet supported)

// Converted from Rust: fn make_rng_from_args(...)
export function make_rng_from_args<I, S>(args: I): Box {
  // Rust variable declaration
  let kind = RngKind.Default;
  // Rust variable declaration
  let seed = 16045690985124838063;
  // Unsupported statement: while let Some (a) = args . next () { let a = a . as_ref () ; if let Some (v) = a . strip_prefix ("--rng=") { if let Ok (k) = RngKind :: from_str (v) { kind = k ; } } else if let Some (v) = a . strip_prefix ("--seed=") { if let Ok (s) = v . parse :: < u64 > () { seed = s ; } } }
  return (undefined as any) /* Unsupported expression: match kind { RngKind :: Default => Box :: new (DefaultRng :: default ()) , RngKind :: Mulberry64 (_) => Box :: new (Mulberry64 :: new (seed)) , RngKind :: Pcg64 (_) => Box :: new (Pcg64 :: new (seed)) , RngKind :: ChaCha8 (_) => Box :: new (ChaCha8Rng :: new (seed)) , } */;
}

// Converted from Rust: fn rng_name_from_args(...)
export function rng_name_from_args<I, S>(args: I): string {
  // Rust variable declaration
  let name = String.from("default");
  // Unsupported statement: while let Some (a) = args . next () { let a = a . as_ref () ; if let Some (v) = a . strip_prefix ("--rng=") { name = v . to_ascii_lowercase () ; } }
  return name;
}

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
  // Unsupported initializer omitted
  return (undefined as any) /* Unsupported expression: Self { x_layers , y_nodes , z_weights , data : vec ! [value ; size] , } */;
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
  return selfObj.index(x, y, z).map((undefined as any) /* Unsupported expression: | i | & self . data [i] */);
}

// Converted from Rust: impl NeuralNetwork<T>::get_mut
function NeuralNetwork_get_mut<T>(selfObj: NeuralNetwork<T>, x: number, y: number, z: number): T | undefined {
  return selfObj.index(x, y, z).map((undefined as any) /* Unsupported expression: move | i | & mut self . data [i] */);
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
  // Unsupported initializer omitted
  // Unsupported statement: f (x , y , z , & self . data [idx])
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
  // Unsupported initializer omitted
  // Unsupported statement: f (x , y , z , & mut self . data [idx])
  }
  }
  }
}

// Converted from Rust: impl NeuralNetwork associated functions
Object.assign(NeuralNetwork, {
  random_uniform: function(x_layers: number, y_nodes: number, z_weights: number, low: number, high: number) {
    function mulberry32Factory(a:number){return function(){let t=a+=0x6D2B79F5;t=Math.imul(t^t>>>15,t|1);t^=t+Math.imul(t^t>>>7,t|61);return ((t^t>>>14)>>>0)/4294967296;}}
    function splitmix64Factory(seed: bigint){
      let state = seed;
      const MASK = (1n<<64n)-1n;
      return function(){
        state = (state + 0x9E3779B97F4A7C15n) & MASK;
        let z = state;
        z ^= z >> 30n; z = (z * 0xBF58476D1CE4E5B9n) & MASK;
        z ^= z >> 27n; z = (z * 0x94D049BB133111EBn) & MASK;
        z ^= z >> 31n;
        const u = Number(z >> 11n) / 9007199254740992; // 2^53
        return u;
      }
    }
    function rotl32(x:number, n:number){ x = x>>>0; return ((x<<n) | (x>>> (32-n)))>>>0; }
    function chacha8Factory(seedU64: bigint){
      const st = new Uint32Array(16);
      st[0]=0x61707865; st[1]=0x3320646e; st[2]=0x79622d32; st[3]=0x6b206574;
      // Mulberry64-like (splitmix64-based) generator to derive key words via f32 cast pattern
      const sm = (function(){ let s=seedU64; const MASK=(1n<<64n)-1n; return function(){ s=(s+0x9E3779B97F4A7C15n)&MASK; let z=s; z^=z>>30n; z=(z*0xBF58476D1CE4E5B9n)&MASK; z^=z>>27n; z=(z*0x94D049BB133111EBn)&MASK; z^=z>>31n; const u=Number(z>>11n)/9007199254740992; return u; };})();
      const F32_MAX = 340282346638528859811704183484516925440.0; // f32::MAX as f64
      for (let i=0;i<8;i++){
        // Emulate sm.next_f32(0.0, f32::MAX) as u32 with saturating cast
        let val = sm()*F32_MAX;
        let u32 = 0; if (!Number.isFinite(val) || val<=0){ u32=0; } else if (val>=4294967295){ u32=4294967295; } else { u32 = Math.floor(val)>>>0; }
        const tweak = Math.imul(0x9E3779B9>>>0, (i+1)>>>0)>>>0;
        st[4+i] = (u32 ^ tweak)>>>0;
      }
      st[12]=0; st[13]=0;
      const seedLo = Number(seedU64 & 0xFFFFFFFFn)>>>0;
      const seedHi = Number((seedU64>>32n) & 0xFFFFFFFFn)>>>0;
      st[14] = (seedLo ^ 0xDEADBEEF)>>>0;
      st[15] = (seedHi ^ 0xBADC0FFE)>>>0;
      const buf = new Uint32Array(16); let idx=16;
      function qr(x:Uint32Array,a:number,b:number,c:number,d:number){ x[a]=(x[a]+x[b])>>>0; x[d]^=x[a]; x[d]=rotl32(x[d],16); x[c]=(x[c]+x[d])>>>0; x[b]^=x[c]; x[b]=rotl32(x[b],12); x[a]=(x[a]+x[b])>>>0; x[d]^=x[a]; x[d]=rotl32(x[d],8); x[c]=(x[c]+x[d])>>>0; x[b]^=x[c]; x[b]=rotl32(x[b],7); }
      function refill(){
        const x = new Uint32Array(st);
        for (let r=0;r<8;r++){
          // column rounds
          qr(x,0,4,8,12); qr(x,1,5,9,13); qr(x,2,6,10,14); qr(x,3,7,11,15);
          // diagonal rounds
          qr(x,0,5,10,15); qr(x,1,6,11,12); qr(x,2,7,8,13); qr(x,3,4,9,14);
        }
        for (let i=0;i<16;i++){ buf[i] = (x[i] + st[i])>>>0; }
        st[12] = (st[12] + 1)>>>0; if (st[12]===0){ st[13] = (st[13]+1)>>>0; }
        idx=0;
      }
      function next_u32(){ if (idx>=16) refill(); const v = buf[idx]; idx++; return v>>>0; }
      return function(){ const r = next_u32() / 4294967296; return r; };
    }
    const g:any = globalThis as any;
    const rngName = (g.__RUST_TO_TS_RNG || '').toString().toLowerCase();
    const seed32 = (g.__RUST_TO_TS_SEED >>> 0) || 0xDEADBEEF;
    const seedU64: bigint = (typeof g.__RUST_TO_TS_SEED_U64 !== 'undefined') ? BigInt(g.__RUST_TO_TS_SEED_U64) : BigInt(seed32 >>> 0);
    let rand: () => number;
    if (rngName === 'mulberry64') { rand = splitmix64Factory(seedU64); }
    else if (rngName === 'pcg64') { rand = splitmix64Factory(seedU64); /* TODO: implement PCG64 */ }
    else if (rngName === 'chacha8' || rngName === 'chacha8rng') { rand = chacha8Factory(seedU64); }
    else { rand = mulberry32Factory(seed32); }
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
  // Unsupported initializer omitted
  // Rust variable declaration
  let rng = rand.thread_rng();
  // Rust variable declaration
  const dist = Uniform.new(low, high);
  // Rust variable declaration
  let data = Vec.with_capacity(size);
  // Unsupported for-loop pattern: for _ in 0 .. size { data . push (dist . sample (& mut rng)) ; } . pat
  // Original: for _ in 0 .. size { data . push (dist . sample (& mut rng)) ; }
  return (undefined as any) /* Unsupported expression: Self { x_layers , y_nodes , z_weights , data , } */;
}

// Converted from Rust: impl NeuralNetwork::random_uniform_f32_with
function NeuralNetwork_random_uniform_f32_with(x_layers: number, y_nodes: number, z_weights: number, low: number, high: number, rng: any): any {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let data = Vec.with_capacity(size);
  // Unsupported for-loop pattern: for _ in 0 .. size { data . push (rng . next_f32 (low , high)) ; } . pat
  // Original: for _ in 0 .. size { data . push (rng . next_f32 (low , high)) ; }
  return (undefined as any) /* Unsupported expression: Self { x_layers , y_nodes , z_weights , data } */;
}

// Converted from Rust: impl NeuralNetwork::random_uniform_f64_with
function NeuralNetwork_random_uniform_f64_with(x_layers: number, y_nodes: number, z_weights: number, low: number, high: number, rng: any): any {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let data = Vec.with_capacity(size);
  // Unsupported for-loop pattern: for _ in 0 .. size { data . push (rng . next_f64 (low , high)) ; } . pat
  // Original: for _ in 0 .. size { data . push (rng . next_f64 (low , high)) ; }
  return (undefined as any) /* Unsupported expression: Self { x_layers , y_nodes , z_weights , data } */;
}

