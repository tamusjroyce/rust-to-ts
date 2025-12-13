import { NeuralNetwork, make_rng_from_args, env, std } from "./lib.ts";

// Converted from Rust: fn main(...)
export function main(): void {
  // Rust variable declaration
  let args = std.env.args().skip(1);
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  const extra_args = args.collect();
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (!target_path.exists()) {
  // Unsupported macro: eprintln!\n  // Original: eprintln ! ("Path not found: {}" , target_path . display ()) ;
  // Rust expression
  std.process.exit(2);
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  const matched = rust_out === ts_out;
  // Rust if
  if (matched) {
  // Rust macro
  console.log(`Outputs match exactly.`);
  } else {
  // Unsupported macro: eprintln!\n  // Original: eprintln ! ("Outputs differ!") ;
  }
  // Rust macro
  console.log(`--- Rust ---
${rust_out}`);
  // Rust macro
  console.log(`--- Deno ---
${ts_out}`);
  // Unsupported statement: std :: process :: exit (if matched { 0 } else { 1 })
}

// Converted from Rust: fn find_example_files(...)
export function find_example_files(root: Path): any | undefined {
  // Rust variable declaration
  let rs_map = Vec.new();
  // Rust variable declaration
  let ts_map = Vec.new();
  // Unsupported statement (unhandled variant)
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if-let
  const __tmp = rs_map.iter().find((undefined as any) /* Unsupported expression: | p | p . file_stem () . and_then (OsStr :: to_str) == Some ("main") */);
  if (__tmp !== undefined) {
    const r_main = __tmp;
  // Rust if-let
  const __tmp = ts_map.iter().find((undefined as any) /* Unsupported expression: | p | p . file_stem () . and_then (OsStr :: to_str) == Some ("main") */);
  if (__tmp !== undefined) {
    const t_main = __tmp;
  // Rust expression
  return Some([r_main.clone(), t_main.clone()]);
  }
  }
  // Unsupported for-loop iterator: & rs_map
  // Original: for r in & rs_map { if let Some (stem) = r . file_stem () . and_then (OsStr :: to_str) { if let Some (t) = ts_map . iter () . find (| t | t . file_stem () . and_then (OsStr :: to_str) == Some (stem)) { return Some ((r . clone () , t . clone ())) ; } } }
  return None;
}

// Converted from Rust: fn run_rust_example(...)
export function run_rust_example(example_root: Path, rs_file: Path, extra_args: any): Result {
  // Rust variable declaration
  const manifest = example_root.join("Cargo.toml");
  // Rust if
  if (manifest.exists()) {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let cmd = Command.new(cargo_cmd);
  // Unsupported statement: cmd . arg ("run") . arg ("--quiet") . arg ("--manifest-path") . arg (& manifest)
  // Rust if
  if (!extra_args.is_empty()) {
  // Rust expression
  cmd.arg("--");
  // Unsupported for-loop iterator: extra_args
  // Original: for a in extra_args { cmd . arg (a) ; }
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (!output.status.success()) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("cargo run failed with status {}\nStderr:\n{}" , output . status , String :: from_utf8_lossy (& output . stderr)) */);
  }
  // Unsupported statement: return Ok (String :: from_utf8_lossy (& output . stdout) . to_string ())
  }
  // Rust if-let
  const __tmp = rs_file.parent().and_then((undefined as any) /* Unsupported expression: | p | p . file_name () */).and_then(OsStr.to_str);
  if (__tmp !== undefined) {
    const parent = __tmp;
  return (undefined as any) /* Unsupported expression: match & * parent . to_ascii_lowercase () { "converter" => { return run_root_bin ("rust-to-ts" , extra_args) ; } "tester" => { return run_root_bin ("tester" , extra_args) ; } _ => { } } */;
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let out_path = PathBuf.from("target/tmp");
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust expression
  out_path.push(exe_name);
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (!status.success()) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("rustc failed with status: {}" , status) */);
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (!output.status.success()) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("Rust example exited with status {}\nStderr:\n{}" , output . status , String :: from_utf8_lossy (& output . stderr)) */);
  }
  return Ok(String.from_utf8_lossy((undefined as any) /* Unsupported expression: & output . stdout */).to_string());
}

// Converted from Rust: fn run_root_bin(...)
export function run_root_bin(bin: string, extra_args: any): Result {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let cmd = Command.new(cargo_cmd);
  // Rust expression
  cmd.arg("run").arg("--quiet").arg("--bin").arg(bin);
  // Rust if
  if (!extra_args.is_empty()) {
  // Rust expression
  cmd.arg("--");
  // Unsupported for-loop iterator: extra_args
  // Original: for a in extra_args { cmd . arg (a) ; }
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (!output.status.success()) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("cargo run --bin {} failed with status {}\nStderr:\n{}" , bin , output . status , String :: from_utf8_lossy (& output . stderr)) */);
  }
  return Ok(String.from_utf8_lossy((undefined as any) /* Unsupported expression: & output . stdout */).to_string());
}

// Converted from Rust: fn run_ts_example(...)
export function run_ts_example(ts_file: Path, extra_args: any): Result {
  return run_with_deno(ts_file, extra_args);
}

// Converted from Rust: fn run_with_deno(...)
export function run_with_deno(ts_file: Path, extra_args: any): Result {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let abs_str = abs.to_string_lossy().to_string();
  // Rust if
  if (abs_str.starts_with("\\?\")) {
  // Unsupported statement: abs_str = abs_str [4 ..] . to_string ()
  }
  // Rust variable declaration
  let file_url = "file:///";
  // Unsupported statement: file_url . push_str (& abs_str . replace ('\\' , "/"))
  // Rust variable declaration
  const tmp_dir = PathBuf.from("target/tmp");
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  const wrapper_path = tmp_dir.join((undefined as any) /* Unsupported macro: format! - Original: format ! ("{}_deno_run.ts" , stem) */);
  // Rust variable declaration
  let js_preamble = String.new();
  // Rust if-let
  const __tmp = extra_args.iter().find((undefined as any) /* Unsupported expression: | a | a . starts_with ("--seed=") */);
  if (__tmp !== undefined) {
    const seed_arg = __tmp;
  // Rust if-let
  const __tmp = seed_arg.splitn(2, '=').nth(1);
  if (__tmp !== undefined) {
    const v = __tmp;
  // Rust if-let
  const __tmp = v.parse();
  if (__tmp !== undefined) {
    const val = __tmp;
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported statement: js_preamble . push_str (& format ! ("(globalThis as any).__RUST_TO_TS_SEED = {} as number;\n" , seed32))
  // Unsupported statement: js_preamble . push_str (& format ! ("(globalThis as any).__RUST_TO_TS_SEED_U64 = BigInt(\"{}\");\n" , seed_val))
  }
  }
  }
  // Rust if-let
  const __tmp = extra_args.iter().find((undefined as any) /* Unsupported expression: | a | a . starts_with ("--rng=") */);
  if (__tmp !== undefined) {
    const rng_arg = __tmp;
  // Rust if-let
  const __tmp = rng_arg.splitn(2, '=').nth(1);
  if (__tmp !== undefined) {
    const v = __tmp;
  // Rust variable declaration
  const name = v.to_ascii_lowercase();
  // Unsupported statement: js_preamble . push_str (& format ! ("(globalThis as any).__RUST_TO_TS_RNG = \"{}\";\n" , name))
  }
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported statement: fs :: write (& wrapper_path , wrapper_code) . map_err (| e | format ! ("failed to write Deno wrapper: {}" , e)) ?
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (!output.status.success()) {
  // Rust expression
  return Err((undefined as any) /* Unsupported macro: format! - Original: format ! ("deno exited with status {}\nStderr:\n{}" , output . status , String :: from_utf8_lossy (& output . stderr)) */);
  }
  return Ok(String.from_utf8_lossy((undefined as any) /* Unsupported expression: & output . stdout */).to_string());
}

// Converted from Rust: fn is_examples_src_converter_main(...)
export function is_examples_src_converter_main(ts_file: Path): boolean {
  // Rust if
  if (ts_file.file_stem().and_then(OsStr.to_str) !== Some("main")) {
  // Rust expression
  return false;
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (parent.file_name().and_then(OsStr.to_str) !== Some("converter")) {
  // Rust expression
  return false;
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (grandparent.file_name().and_then(OsStr.to_str) !== Some("src")) {
  // Rust expression
  return false;
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  return gg.file_name().and_then(OsStr.to_str) === Some("Examples");
}

