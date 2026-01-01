import { NeuralNetwork, make_rng_from_args, env, std } from "./lib.ts";

// Converted from Rust: fn main(...)
export function main(): void {
  // Rust variable declaration
  let args = env.args().skip(1);
  // Rust variable declaration
  let dry_run = false;
  // Rust variable declaration
  let output_dir = None;
  // Rust variable declaration
  let inputs = Vec.new();
  // Unsupported statement: while let Some (arg) = args . next () { match arg . as_str () { "--dry-run" => dry_run = true , "--output-dir" => { let val = args . next () . unwrap_or_else (| | { eprintln ! ("--output-dir requires a path argument") ; std :: process :: exit (1) ; }) ; output_dir = Some (PathBuf :: from (val)) ; } _ => inputs . push (PathBuf :: from (arg)) , } }
  // Rust if
  if (inputs.is_empty()) {
  // Unsupported macro: eprintln!\n  // Original: eprintln ! ("Usage: ast-v2 [--dry-run] [--output-dir DIR] <path> [more paths...]") ;
  // Unsupported macro: eprintln!\n  // Original: eprintln ! ("  <path> can be a file or directory, e.g. Examples/HelloWorld") ;
  // Rust expression
  std.process.exit(2);
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (!dry_run) {
  // Rust if-let
  const __tmp = fs.create_dir_all((undefined as any) /* Unsupported expression: & output_dir */);
  if (__tmp !== undefined) {
    const val = __tmp;
  // Unsupported macro: eprintln!\n  // Original: eprintln ! ("Failed to create output dir {}: {}" , output_dir . display () , e) ;
  // Rust expression
  std.process.exit(1);
  }
  }
  // Rust variable declaration
  let had_error = false;
  // Unsupported for-loop iterator: & inputs
  // Original: for input in & inputs { if let Err (e) = process_path (input , & output_dir , dry_run) { eprintln ! ("{}" , e) ; had_error = true ; } }
  // Rust if
  if (had_error) {
  // Rust expression
  std.process.exit(1);
  }
}

// Converted from Rust: fn process_path(...)
export function process_path(path: Path, output_dir: Path, dry_run: boolean): Result {
  // Rust if
  if (path.starts_with(output_dir)) {
  // Rust expression
  return Ok([]);
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (metadata.is_dir()) {
  // Rust if
  if (path === output_dir) {
  // Rust expression
  return Ok([]);
  }
  // Unsupported for-loop iterator: fs :: read_dir (path) . map_err (| e | format ! ("Failed to read dir {}: {}" , path . display () , e)) ?
  // Original: for entry in fs :: read_dir (path) . map_err (| e | format ! ("Failed to read dir {}: {}" , path . display () , e)) ? { let entry = entry . map_err (| e | format ! ("Failed to read dir entry in {}: {}" , path . display () , e)) ? ; let child = entry . path () ; process_path (& child , output_dir , dry_run) ? ; }
  // Rust expression
  return Ok([]);
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (ext !== "rs" && ext !== "ts") {
  // Rust expression
  return Ok([]);
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  let dest_rel = rel;
  // Unsupported statement: dest_rel . set_extension (if ext == "rs" { "ts" } else { "rs" })
  // Rust variable declaration
  const dest_path = output_dir.join(dest_rel);
  // Rust if
  if (dry_run) {
  // Rust macro
  console.log(`--dry-run: would convert ${path.display()} -> ${dest_path.display()}`);
  // Rust expression
  return Ok([]);
  }
  // Rust if-let
  const __tmp = dest_path.parent();
  if (__tmp !== undefined) {
    const parent = __tmp;
  // Unsupported statement: fs :: create_dir_all (parent) . map_err (| e | format ! ("Failed to create dir {}: {}" , parent . display () , e)) ?
  }
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported statement: fs :: write (& dest_path , converted) . map_err (| e | format ! ("Failed to write {}: {}" , dest_path . display () , e)) ?
  // Rust macro
  console.log(`Wrote ${dest_path.display()}`);
  return Ok([]);
}

