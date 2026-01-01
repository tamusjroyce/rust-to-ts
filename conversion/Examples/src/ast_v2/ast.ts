// Converted from Rust: struct Field
interface Field {
  name: string;
  ty: TypeRef;
}

// Converted from Rust: struct TypeDecl
interface TypeDecl {
  name: string;
  kind: TypeKind;
  fields: Vec;
}

// Converted from Rust: struct Param
interface Param {
  name: string;
  ty: TypeRef;
}

// Converted from Rust: struct Function
interface Function {
  name: string;
  params: Vec;
  return_type: Option;
  kind: FunctionKind;
}

// Converted from Rust: struct Module
interface Module {
  name: string;
  types: Vec;
  functions: Vec;
}

