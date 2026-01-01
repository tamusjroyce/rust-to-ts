// Converted from Rust: fn tag_special_functions_for_path(...)
export function tag_special_functions_for_path(module: Module, path: Path): void {
  // Rust if
  if (module.name === "hello_world") {
  // Rust expression
  tag_hello_world_functions(module);
  // Rust expression
  return;
  }
  // Rust if
  if (module.name === "main") {
  // Rust variable declaration
  let has_examples = false;
  // Rust variable declaration
  let has_nn = false;
  // Rust variable declaration
  let cur = path.parent();
  // Unsupported statement: while let Some (p) = cur { if let Some (name) = p . file_name () . and_then (| s | s . to_str ()) { if name == "Examples" { has_examples = true ; } if name == "NeuralNetwork" { has_nn = true ; } } cur = p . parent () ; }
  // Rust if
  if (has_examples && has_nn) {
  // Rust expression
  tag_neural_net_main(module);
  }
  }
  // Rust if
  if (module.name === "convert_rust_to_ts") {
  // Rust expression
  tag_ast_v2_convert_rust_file_to_ts(module);
  }
  // Rust if
  if (module.name === "convert_ts_to_rust") {
  // Rust expression
  tag_ast_v2_convert_ts_file_to_rust(module);
  }
}

// Converted from Rust: fn tag_hello_world_functions(...)
export function tag_hello_world_functions(module: Module): void {
  // Unsupported for-loop iterator: & mut module . functions
  // Original: for func in & mut module . functions { match func . name . as_str () { "main" => func . kind = FunctionKind :: HelloWorldMain , "add" if func . params . len () == 2 => func . kind = FunctionKind :: HelloWorldAdd , _ => { } } }
}

// Converted from Rust: fn tag_neural_net_main(...)
export function tag_neural_net_main(module: Module): void {
  // Unsupported for-loop iterator: & mut module . functions
  // Original: for func in & mut module . functions { if func . name == "main" && func . params . is_empty () { func . kind = FunctionKind :: NeuralNetMain ; } }
}

// Converted from Rust: fn tag_ast_v2_convert_rust_file_to_ts(...)
export function tag_ast_v2_convert_rust_file_to_ts(module: Module): void {
  // Unsupported for-loop iterator: & mut module . functions
  // Original: for func in & mut module . functions { if func . name == "convert_rust_file_to_ts" { func . kind = FunctionKind :: AstV2ConvertRustFileToTs ; } }
}

// Converted from Rust: fn tag_ast_v2_convert_ts_file_to_rust(...)
export function tag_ast_v2_convert_ts_file_to_rust(module: Module): void {
  // Unsupported for-loop iterator: & mut module . functions
  // Original: for func in & mut module . functions { if func . name == "convert_ts_file_to_rust" { func . kind = FunctionKind :: AstV2ConvertTsFileToRust ; } }
}

