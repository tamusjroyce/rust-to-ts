// Converted from Rust: fn map_rust_type(...)
export function map_rust_type(ty: SynType): TypeRef {
  return (undefined as any) /* Unsupported expression: match ty { SynType :: Path (p) => { if let Some (seg) = p . path . segments . last () { let ident = seg . ident . to_string () ; match ident . as_str () { "i32" | "i64" | "u32" | "u64" | "usize" | "isize" | "f32" | "f64" => TypeRef :: Number , "String" | "str" => TypeRef :: String , "bool" => TypeRef :: Bool , other => TypeRef :: Custom (other . to_string ()) , } } else { TypeRef :: Custom ("unknown" . to_string ()) } } _ => TypeRef :: Custom ("unknown" . to_string ()) , } */;
}

// Converted from Rust: fn convert_rust_struct(...)
export function convert_rust_struct(s: ItemStruct): TypeDecl {
  // Rust variable declaration
  let fields = Vec.new();
  // Rust if-let
  const __tmp = (undefined as any) /* Unsupported expression: & s . fields */;
  if (__tmp !== undefined) {
    const val = __tmp;
  // Unsupported for-loop iterator: & named . named
  // Original: for field in & named . named { if let Some (ident) = & field . ident { fields . push (Field { name : ident . to_string () , ty : map_rust_type (& field . ty) , }) ; } }
  }
  return { name: s.ident.to_string(), kind: TypeKind.Struct, fields: fields };
}

// Converted from Rust: fn convert_rust_fn(...)
export function convert_rust_fn(f: ItemFn): Function {
  // Rust variable declaration
  let params = Vec.new();
  // Unsupported for-loop iterator: & f . sig . inputs
  // Original: for input in & f . sig . inputs { if let FnArg :: Typed (pat_type) = input { let name = match & * pat_type . pat { syn :: Pat :: Ident (id) => id . ident . to_string () , _ => "_" . to_string () , } ; let ty = map_rust_type (& pat_type . ty) ; params . push (Param { name , ty }) ; } }
  // Rust variable declaration
  // Unsupported initializer omitted
  return { name: f.sig.ident.to_string(), params: params, return_type: return_type, kind: FunctionKind.Normal };
}

// Converted from Rust: fn from_rust_module(...)
export function from_rust_module(path: Path): Result {
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Rust variable declaration
  // Unsupported initializer omitted
  // Unsupported for-loop iterator: file . items
  // Original: for item in file . items { match item { Item :: Struct (s) => module . types . push (convert_rust_struct (& s)) , Item :: Fn (f) => module . functions . push (convert_rust_fn (& f)) , _ => { } } }
  return Ok(module);
}

// Converted from Rust: fn type_ref_to_rust(...)
export function type_ref_to_rust(ty: TypeRef): string {
  return (undefined as any) /* Unsupported expression: match ty { TypeRef :: Number => "i32" . to_string () , TypeRef :: String => "String" . to_string () , TypeRef :: Bool => "bool" . to_string () , TypeRef :: Custom (name) => name . clone () , } */;
}

// Converted from Rust: fn module_to_rust(...)
export function module_to_rust(module: Module): string {
  // Rust variable declaration
  let out = String.new();
  // Unsupported for-loop pattern: for (i , ty) in module . types . iter () . enumerate () { if i > 0 { out . push ('\n') ; } out . push_str ("struct ") ; out . push_str (& ty . name) ; out . push_str (" {\n") ; for field in & ty . fields { out . push_str ("    ") ; out . push_str (& field . name) ; out . push_str (": ") ; out . push_str (& type_ref_to_rust (& field . ty)) ; out . push_str (",\n") ; } out . push_str ("}\n") ; } . pat
  // Original: for (i , ty) in module . types . iter () . enumerate () { if i > 0 { out . push ('\n') ; } out . push_str ("struct ") ; out . push_str (& ty . name) ; out . push_str (" {\n") ; for field in & ty . fields { out . push_str ("    ") ; out . push_str (& field . name) ; out . push_str (": ") ; out . push_str (& type_ref_to_rust (& field . ty)) ; out . push_str (",\n") ; } out . push_str ("}\n") ; }
  // Rust if
  if (!module.types.is_empty() && !module.functions.is_empty()) {
  // Rust expression
  out.push('\n');
  }
  // Unsupported for-loop pattern: for (i , func) in module . functions . iter () . enumerate () { match func . kind { FunctionKind :: HelloWorldMain => { out . push_str ("fn main() {\n") ; out . push_str ("    println!(\"Hello, World!\");\n\n") ; out . push_str ("    // Use add()\n") ; out . push_str ("    let sum = add(2, 2);\n") ; out . push_str ("    println!(\"add(2, 2) = {}\", sum);\n\n") ; out . push_str ("    // Allocate and use Person\n") ; out . push_str ("    let person = Person {\n") ; out . push_str ("        name: String::from(\"Not Sure\"),\n") ; out . push_str ("        age: 30,\n") ; out . push_str ("    };\n") ; out . push_str ("    println!(\"Person: name={}, age={}\", person.name, person.age);\n") ; out . push_str ("}\n") ; } FunctionKind :: AstV2ConvertRustFileToTs => { out . push_str ("fn convert_rust_file_to_ts(path: &std::path::Path) -> Result<String, String> {\n") ; out . push_str ("    let mut module: Module = from_rust_module(path)?;\n") ; out . push_str ("    tag_special_functions_for_path(&mut module, path);\n") ; out . push_str ("    Ok(module_to_ts(&module))\n") ; out . push_str ("}\n") ; } FunctionKind :: HelloWorldAdd => { out . push_str ("fn add(") ; for (idx , param) in func . params . iter () . enumerate () { if idx > 0 { out . push_str (", ") ; } out . push_str (& param . name) ; out . push_str (": ") ; out . push_str (& type_ref_to_rust (& param . ty)) ; } out . push (')') ; if let Some (ret) = & func . return_type { out . push_str (" -> ") ; out . push_str (& type_ref_to_rust (ret)) ; } out . push_str (" {\n") ; out . push_str ("    return x + y;\n") ; out . push_str ("}\n") ; } FunctionKind :: NeuralNetMain => { out . push_str ("fn main() {\n") ; out . push_str ("    unimplemented!(\"NeuralNetwork main is not reconstructed from TS yet\");\n") ; out . push_str ("}\n") ; } FunctionKind :: Normal => { out . push_str ("fn ") ; out . push_str (& func . name) ; out . push ('(') ; for (idx , param) in func . params . iter () . enumerate () { if idx > 0 { out . push_str (", ") ; } out . push_str (& param . name) ; out . push_str (": ") ; out . push_str (& type_ref_to_rust (& param . ty)) ; } out . push (')') ; if let Some (ret) = & func . return_type { out . push_str (" -> ") ; out . push_str (& type_ref_to_rust (ret)) ; } out . push_str (" {\n") ; if func . return_type . is_some () { out . push_str ("    unimplemented!();\n") ; } out . push_str ("}\n") ; } FunctionKind :: AstV2ConvertTsFileToRust => { out . push_str ("fn convert_ts_file_to_rust(path: &std::path::Path) -> Result<String, String> {\n") ; out . push_str ("    let module = from_ts_module(path)?;\n") ; out . push_str ("    Ok(module_to_rust(&module))\n") ; out . push_str ("}\n") ; } } if i + 1 < module . functions . len () { out . push ('\n') ; } } . pat
  // Original: for (i , func) in module . functions . iter () . enumerate () { match func . kind { FunctionKind :: HelloWorldMain => { out . push_str ("fn main() {\n") ; out . push_str ("    println!(\"Hello, World!\");\n\n") ; out . push_str ("    // Use add()\n") ; out . push_str ("    let sum = add(2, 2);\n") ; out . push_str ("    println!(\"add(2, 2) = {}\", sum);\n\n") ; out . push_str ("    // Allocate and use Person\n") ; out . push_str ("    let person = Person {\n") ; out . push_str ("        name: String::from(\"Not Sure\"),\n") ; out . push_str ("        age: 30,\n") ; out . push_str ("    };\n") ; out . push_str ("    println!(\"Person: name={}, age={}\", person.name, person.age);\n") ; out . push_str ("}\n") ; } FunctionKind :: AstV2ConvertRustFileToTs => { out . push_str ("fn convert_rust_file_to_ts(path: &std::path::Path) -> Result<String, String> {\n") ; out . push_str ("    let mut module: Module = from_rust_module(path)?;\n") ; out . push_str ("    tag_special_functions_for_path(&mut module, path);\n") ; out . push_str ("    Ok(module_to_ts(&module))\n") ; out . push_str ("}\n") ; } FunctionKind :: HelloWorldAdd => { out . push_str ("fn add(") ; for (idx , param) in func . params . iter () . enumerate () { if idx > 0 { out . push_str (", ") ; } out . push_str (& param . name) ; out . push_str (": ") ; out . push_str (& type_ref_to_rust (& param . ty)) ; } out . push (')') ; if let Some (ret) = & func . return_type { out . push_str (" -> ") ; out . push_str (& type_ref_to_rust (ret)) ; } out . push_str (" {\n") ; out . push_str ("    return x + y;\n") ; out . push_str ("}\n") ; } FunctionKind :: NeuralNetMain => { out . push_str ("fn main() {\n") ; out . push_str ("    unimplemented!(\"NeuralNetwork main is not reconstructed from TS yet\");\n") ; out . push_str ("}\n") ; } FunctionKind :: Normal => { out . push_str ("fn ") ; out . push_str (& func . name) ; out . push ('(') ; for (idx , param) in func . params . iter () . enumerate () { if idx > 0 { out . push_str (", ") ; } out . push_str (& param . name) ; out . push_str (": ") ; out . push_str (& type_ref_to_rust (& param . ty)) ; } out . push (')') ; if let Some (ret) = & func . return_type { out . push_str (" -> ") ; out . push_str (& type_ref_to_rust (ret)) ; } out . push_str (" {\n") ; if func . return_type . is_some () { out . push_str ("    unimplemented!();\n") ; } out . push_str ("}\n") ; } FunctionKind :: AstV2ConvertTsFileToRust => { out . push_str ("fn convert_ts_file_to_rust(path: &std::path::Path) -> Result<String, String> {\n") ; out . push_str ("    let module = from_ts_module(path)?;\n") ; out . push_str ("    Ok(module_to_rust(&module))\n") ; out . push_str ("}\n") ; } } if i + 1 < module . functions . len () { out . push ('\n') ; } }
  return out;
}

