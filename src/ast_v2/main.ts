import { NeuralNetwork, make_rng_from_args, env, std } from "./lib.ts";

// Unsupported Rust item: # [derive (Clone , Copy , Debug , PartialEq , Eq)] enum Direction { RsToTs , TsToRs , Both , }

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
  // Rust variable declaration
  let direction = Direction.RsToTs;
  // Unsupported statement: while let Some (arg) = args . next () { match arg . as_str () { "--dry-run" => dry_run = true , "--output-dir" => { let val = args . next () . unwrap_or_else (| | { eprintln ! ("--output-dir requires a path argument") ; std :: process :: exit (1) ; }) ; output_dir = Some (PathBuf :: from (val)) ; } "--ts-to-rs" => direction = Direction :: TsToRs , "--both" => direction = Direction :: Both , _ => inputs . push (PathBuf :: from (arg)) , } }
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
  // Original: for input in & inputs { if let Err (e) = process_path (input , & output_dir , dry_run , direction) { eprintln ! ("{}" , e) ; had_error = true ; } }
  // Rust if
  if (had_error) {
  // Rust expression
  std.process.exit(1);
  }
}

// Converted from Rust: fn process_path(...)
export function process_path(path: Path, output_dir: Path, dry_run: boolean, direction: Direction): Result {
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
  // Original: for entry in fs :: read_dir (path) . map_err (| e | format ! ("Failed to read dir {}: {}" , path . display () , e)) ? { let entry = entry . map_err (| e | format ! ("Failed to read dir entry in {}: {}" , path . display () , e)) ? ; let child = entry . path () ; process_path (& child , output_dir , dry_run , direction) ? ; }
  // Rust expression
  return Ok([]);
  }
  // Rust if-let
  const __tmp = path.file_name().and_then((undefined as any) /* Unsupported expression: | s | s . to_str () */);
  if (__tmp !== undefined) {
    const file_name = __tmp;
  // Rust if
  if (file_name === "Cargo.toml") {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (dry_run) {
  // Rust macro
  console.log(`--dry-run: would mirror Cargo.toml ${path.display()} -> ${dest.display()}`);
  // Rust expression
  return Ok([]);
  }
  // Rust if-let
  const __tmp = dest.parent();
  if (__tmp !== undefined) {
    const parent = __tmp;
  // Unsupported statement: fs :: create_dir_all (parent) . map_err (| e | format ! ("Failed to create dir {}: {}" , parent . display () , e)) ?
  }
  // Unsupported statement: fs :: copy (path , & dest) . map_err (| e | { format ! ("Failed to copy Cargo.toml {} -> {}: {}" , path . display () , dest . display () , e) }) ?
  // Rust expression
  return Ok([]);
  }
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
  let dest_rel = rel.clone();
  // Unsupported statement: dest_rel . set_extension (if ext == "rs" { "ts" } else { "rs" })
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust if
  if (dry_run) {
  // Unsupported statement: match (ext . as_str () , direction) { ("rs" , Direction :: RsToTs) | ("rs" , Direction :: Both) => { println ! ("--dry-run: would convert {} -> {}" , path . display () , dest_path . display ()) ; } ("ts" , Direction :: TsToRs) | ("ts" , Direction :: Both) => { println ! ("--dry-run: would convert {} -> {}" , path . display () , dest_path . display ()) ; } _ => { println ! ("--dry-run: would mirror source {} into {}" , path . display () , src_mirror_path . display ()) ; } }
  // Rust expression
  return Ok([]);
  }
  // Rust if-let
  const __tmp = src_mirror_path.parent();
  if (__tmp !== undefined) {
    const parent = __tmp;
  // Unsupported statement: fs :: create_dir_all (parent) . map_err (| e | format ! ("Failed to create dir {}: {}" , parent . display () , e)) ?
  }
  // Unsupported statement: fs :: copy (path , & src_mirror_path) . map_err (| e | format ! ("Failed to copy {} -> {}: {}" , path . display () , src_mirror_path . display () , e)) ?
  // Unsupported statement: match (ext . as_str () , direction) { ("rs" , Direction :: RsToTs) | ("rs" , Direction :: Both) => { if let Some (parent) = dest_path . parent () { fs :: create_dir_all (parent) . map_err (| e | format ! ("Failed to create dir {}: {}" , parent . display () , e)) ? ; } let converted = convert_rust_file_to_ts (path) ? ; fs :: write (& dest_path , converted) . map_err (| e | format ! ("Failed to write {}: {}" , dest_path . display () , e)) ? ; println ! ("Wrote {}" , dest_path . display ()) ; } ("ts" , Direction :: TsToRs) | ("ts" , Direction :: Both) => { if let Some (parent) = dest_path . parent () { fs :: create_dir_all (parent) . map_err (| e | format ! ("Failed to create dir {}: {}" , parent . display () , e)) ? ; } let converted = convert_ts_file_to_rust (path) ? ; fs :: write (& dest_path , converted) . map_err (| e | format ! ("Failed to write {}: {}" , dest_path . display () , e)) ? ; println ! ("Wrote {}" , dest_path . display ()) ; } _ => { } }
  return Ok([]);
}

