// Unsupported Rust item: # [derive (Debug , Clone , PartialEq , Eq)] pub enum TypeKind { Struct , Interface , }

// Unsupported Rust item: # [derive (Debug , Clone , PartialEq , Eq)] pub enum TypeRef { Number , String , Bool , Custom (String) , }

// Converted from Rust: struct Field
interface Field {
  name: string;
  ty: TypeRef;
}

// Converted from Rust: struct TypeDecl
interface TypeDecl {
  name: string;
  kind: TypeKind;
  fields: Field[];
}

// Converted from Rust: struct Param
interface Param {
  name: string;
  ty: TypeRef;
}

// Unsupported Rust item: # [derive (Debug , Clone , PartialEq , Eq)] pub enum FunctionKind { Normal , HelloWorldMain , HelloWorldAdd , NeuralNetMain , AstV2ConvertRustFileToTs , AstV2ConvertTsFileToRust , }

// Converted from Rust: struct Function
interface Function {
  name: string;
  params: Param[];
  return_type: TypeRef | undefined;
  kind: FunctionKind;
}

// Converted from Rust: struct Module
interface Module {
  name: string;
  types: TypeDecl[];
  functions: Function[];
}

// Unsupported Rust item: # [allow (dead_code)] pub type TagFn = fn (& mut Module , & Path) ;

