import { NeuralNetwork, make_rng_from_args, env, std } from "./lib.ts";

// Converted from Rust: fn main(...)
export function main(): void {
  // Rust variable declaration
  const args = env.args().skip(1).collect();
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let total = 0;
  // Rust variable declaration
  let had_error = false;
  // Unsupported for-loop iterator: roots
  // Original: for root in roots { match converter :: convert_rs_dir_to_ts_side_by_side (& root) { Ok (paths) => { for p in & paths { println ! ("Wrote {}" , p . display ()) ; } total += paths . len () ; } Err (e) => { eprintln ! ("Error converting {}: {}" , root . display () , e) ; had_error = true ; } } }
  // Rust macro
  console.log(`Converted ${total} file(s).`);
  // Rust if
  if (had_error) {
  // Rust expression
  std.process.exit(1);
  }
}

