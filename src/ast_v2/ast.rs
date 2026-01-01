use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Struct,
    Interface,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    Number,
    String,
    Bool,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDecl {
    pub name: String,
    pub kind: TypeKind,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub ty: TypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionKind {
    Normal,
    HelloWorldMain,
    HelloWorldAdd,
    NeuralNetMain,
    AstV2ConvertRustFileToTs,
    AstV2ConvertTsFileToRust,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeRef>,
    pub kind: FunctionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: String,
    pub types: Vec<TypeDecl>,
    pub functions: Vec<Function>,
}

// Utility hook for tagging special functions can be reused by
// language-specific adapters if they need the file path.
#[allow(dead_code)]
pub type TagFn = fn(&mut Module, &Path);
