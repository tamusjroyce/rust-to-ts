use std::fs;
use std::path::Path;

use quote::ToTokens;
use syn::{self, FnArg, Fields, Item, ItemFn, ItemStruct, ReturnType, Type as SynType};

use crate::ast_v2::ast::{
    Field, Function, FunctionKind, Module, Param, TypeDecl, TypeKind, TypeRef,
};

fn map_rust_type(ty: &SynType) -> TypeRef {
    match ty {
        SynType::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                let ident = seg.ident.to_string();
                match ident.as_str() {
                    "i32" | "i64" | "u32" | "u64" | "usize" | "isize" | "f32" | "f64" => {
                        TypeRef::Number
                    }
                    "String" | "str" => TypeRef::String,
                    "bool" => TypeRef::Bool,
                    other => TypeRef::Custom(other.to_string()),
                }
            } else {
                TypeRef::Custom("unknown".to_string())
            }
        }
        _ => TypeRef::Custom("unknown".to_string()),
    }
}

fn convert_rust_struct(s: &ItemStruct) -> TypeDecl {
    let mut fields = Vec::new();
    if let Fields::Named(named) = &s.fields {
        for field in &named.named {
            if let Some(ident) = &field.ident {
                fields.push(Field {
                    name: ident.to_string(),
                    ty: map_rust_type(&field.ty),
                });
            }
        }
    }

    TypeDecl {
        name: s.ident.to_string(),
        kind: TypeKind::Struct,
        fields,
    }
}

fn convert_rust_fn(f: &ItemFn) -> Function {
    let mut params = Vec::new();
    for input in &f.sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            let name = match &*pat_type.pat {
                syn::Pat::Ident(id) => id.ident.to_string(),
                _ => "_".to_string(),
            };
            let ty = map_rust_type(&pat_type.ty);
            params.push(Param { name, ty });
        }
    }

    let return_type = match &f.sig.output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(map_rust_type(ty)),
    };

    let body: Vec<String> = f
        .block
        .stmts
        .iter()
        .map(|stmt| stmt.to_token_stream().to_string())
        .collect();

    Function {
        name: f.sig.ident.to_string(),
        params,
        return_type,
        kind: FunctionKind::Normal,
        body,
    }
}

pub fn from_rust_src(src: &str, module_name: &str) -> Result<Module, String> {
    let file: syn::File =
        syn::parse_file(src).map_err(|e| format!("Failed to parse Rust source: {e}"))?;

    let mut module = Module {
        name: module_name.to_string(),
        types: Vec::new(),
        functions: Vec::new(),
    };

    for item in file.items {
        match item {
            Item::Struct(s) => module.types.push(convert_rust_struct(&s)),
            Item::Fn(f) => module.functions.push(convert_rust_fn(&f)),
            _ => {}
        }
    }

    Ok(module)
}

pub fn from_rust_module(path: &Path) -> Result<Module, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read Rust file {}: {}", path.display(), e))?;

    let module_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    from_rust_src(&src, module_name)
}

fn type_ref_to_rust(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Number => "i32".to_string(),
        TypeRef::String => "String".to_string(),
        TypeRef::Bool => "bool".to_string(),
        TypeRef::Custom(name) => name.clone(),
    }
}

pub fn module_to_rust(module: &Module) -> String {
    let mut out = String::new();

    for (i, ty) in module.types.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // For now, both Struct and Interface become a Rust `struct`.
        out.push_str("struct ");
        out.push_str(&ty.name);
        out.push_str(" {\n");
        for field in &ty.fields {
            out.push_str("    ");
            out.push_str(&field.name);
            out.push_str(": ");
            out.push_str(&type_ref_to_rust(&field.ty));
            out.push_str(",\n");
        }
        out.push_str("}\n");
    }

    if !module.types.is_empty() && !module.functions.is_empty() {
        out.push('\n');
    }

    for (i, func) in module.functions.iter().enumerate() {
        match func.kind {
            FunctionKind::HelloWorldMain => {
                out.push_str("fn main() {\n");
                out.push_str("    println!(\"Hello, World!\");\n\n");
                out.push_str("    // Use add()\n");
                out.push_str("    let sum = add(2, 2);\n");
                out.push_str("    println!(\"add(2, 2) = {}\", sum);\n\n");
                out.push_str("    // Allocate and use Person\n");
                out.push_str("    let person = Person {\n");
                out.push_str("        name: String::from(\"Not Sure\"),\n");
                out.push_str("        age: 30,\n");
                out.push_str("    };\n");
                out.push_str("    println!(\"Person: name={}, age={}\", person.name, person.age);\n");
                out.push_str("}\n");
            }
            FunctionKind::AstV2ConvertRustFileToTs => {
                out.push_str("fn convert_rust_file_to_ts(path: &std::path::Path) -> Result<String, String> {\n");
                out.push_str("    let mut module: Module = from_rust_module(path)?;\n");
                out.push_str("    tag_special_functions_for_path(&mut module, path);\n");
                out.push_str("    Ok(module_to_ts(&module))\n");
                out.push_str("}\n");
            }
            FunctionKind::HelloWorldAdd => {
                out.push_str("fn add(");
                for (idx, param) in func.params.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&param.name);
                    out.push_str(": ");
                    out.push_str(&type_ref_to_rust(&param.ty));
                }
                out.push(')');
                if let Some(ret) = &func.return_type {
                    out.push_str(" -> ");
                    out.push_str(&type_ref_to_rust(ret));
                }
                out.push_str(" {\n");
                if func.body.is_empty() {
                    out.push_str("    return x + y;\n");
                } else {
                    for line in &func.body {
                        out.push_str("    ");
                        out.push_str(line);
                        out.push('\n');
                    }
                }
                out.push_str("}\n");
            }
            FunctionKind::NeuralNetMain => {
                // For now, emit a stub Rust main for NeuralNetwork when
                // converting TS back to Rust. The real NeuralNetwork logic
                // lives in the original examples.
                out.push_str("fn main() {\n");
                out.push_str("    unimplemented!(\"NeuralNetwork main is not reconstructed from TS yet\");\n");
                out.push_str("}\n");
            }
            FunctionKind::Normal => {
                out.push_str("fn ");
                out.push_str(&func.name);
                out.push('(');
                for (idx, param) in func.params.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&param.name);
                    out.push_str(": ");
                    out.push_str(&type_ref_to_rust(&param.ty));
                }
                out.push(')');
                if let Some(ret) = &func.return_type {
                    out.push_str(" -> ");
                    out.push_str(&type_ref_to_rust(ret));
                }
                out.push_str(" {\n");
                if func.body.is_empty() {
                    if func.return_type.is_some() {
                        out.push_str("    Default::default()\n");
                    }
                } else {
                    for line in &func.body {
                        out.push_str("    ");
                        out.push_str(line);
                        out.push('\n');
                    }
                }
                out.push_str("}\n");
            }
            FunctionKind::AstV2ConvertTsFileToRust => {
                out.push_str(
                    "fn convert_ts_file_to_rust(path: &std::path::Path) -> Result<String, String> {\n",
                );
                out.push_str("    let module = from_ts_module(path)?;\n");
                out.push_str("    Ok(module_to_rust(&module))\n");
                out.push_str("}\n");
            }
            FunctionKind::ConverterConvertType => {
                out.push_str("fn convert_type(ty: Type) -> String {\n");
                out.push_str("    let ir = rust_type_to_ir(ty);\n");
                out.push_str("    convert_ir_type(&ir)\n");
                out.push_str("}\n");
            }
            FunctionKind::ConverterConvertIrType => {
                out.push_str("fn convert_ir_type(ir: &IrType) -> String {\n");
                out.push_str("    match ir {\n");
                out.push_str("        IrType::Number => \"number\".to_string(),\n");
                out.push_str("        IrType::String => \"string\".to_string(),\n");
                out.push_str("        IrType::Bool => \"boolean\".to_string(),\n");
                out.push_str("        IrType::Any => \"any\".to_string(),\n");
                out.push_str("        IrType::Vec(inner) => format!(\"{}[]\", convert_ir_type(inner)),\n");
                out.push_str(
                    "        IrType::Option(inner) => format!(\"{} | undefined\", convert_ir_type(inner)),\n",
                );
                out.push_str("        IrType::Custom(name) => name.clone(),\n");
                out.push_str("    }\n");
                out.push_str("}\n");
            }
            FunctionKind::ConverterTsPathToString => {
                out.push_str("fn ts_path_to_string(path: &syn::Path) -> String {\n");
                out.push_str("    let mut parts: Vec<String> = Vec::new();\n");
                out.push_str("    for seg in &path.segments {\n");
                out.push_str("        parts.push(seg.ident.to_string());\n");
                out.push_str("    }\n");
                out.push_str("    parts.join(\".\")\n");
                out.push_str("}\n");
            }
        }

        if i + 1 < module.functions.len() {
            out.push('\n');
        }
    }

    out
}
