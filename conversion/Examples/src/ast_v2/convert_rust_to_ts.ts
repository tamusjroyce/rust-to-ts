export function convert_rust_file_to_ts(path: unknown): Result {
  // Delegate to the AST-based pipeline used in the TS self-example:
  // Rust source at `path` → Module (from_rust_module) → tag → TS string.
  const module: Module = from_rust_module(path as any);
  tag_special_functions_for_path(module, path as any);
  return Ok(module_to_ts(module));
}
