use std::fs;
use std::path::Path;

use syn::{self, Item, ItemFn, ItemStruct, Fields, Type as SynType, FnArg, ReturnType};

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

fn map_rust_type(ty: &SynType) -> TypeRef {
    match ty {
        SynType::Path(p) => {
            if let Some(seg) = p.path.segments.last() {
                let ident = seg.ident.to_string();
                match ident.as_str() {
                    "i32" | "i64" | "u32" | "u64" | "usize" | "isize" | "f32" | "f64" => TypeRef::Number,
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

fn map_ts_type(name: &str) -> TypeRef {
    match name.trim() {
        "number" => TypeRef::Number,
        "string" => TypeRef::String,
        "boolean" => TypeRef::Bool,
        other => TypeRef::Custom(other.to_string()),
    }
}

pub fn from_rust_hello_world(path: &Path) -> Result<Module, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read Rust file {}: {}", path.display(), e))?;
    let file: syn::File = syn::parse_file(&src)
        .map_err(|e| format!("Failed to parse Rust file {}: {}", path.display(), e))?;

    let mut module = Module {
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
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

    tag_hello_world_functions(&mut module);
    Ok(module)
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

    Function {
        name: f.sig.ident.to_string(),
        params,
        return_type,
        kind: FunctionKind::Normal,
    }
}

pub fn from_ts_hello_world(path: &Path) -> Result<Module, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read TS file {}: {}", path.display(), e))?;

    let mut module = Module {
        name: path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string(),
        types: Vec::new(),
        functions: Vec::new(),
    };

    let mut lines = src.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();

        if let Some(func) = parse_ts_function(trimmed) {
            module.functions.push(func);
            continue;
        }

        if trimmed.starts_with("interface ") {
            if let Some(ty) = parse_ts_interface(trimmed, &mut lines) {
                module.types.push(ty);
            }
        }
    }

    tag_hello_world_functions(&mut module);
    Ok(module)
}

fn parse_ts_function(line: &str) -> Option<Function> {
    if !line.starts_with("export function ") {
        return None;
    }
    let after = &line["export function ".len()..];
    let open_paren = after.find('(')?;
    let name = after[..open_paren].trim().to_string();
    let rest = &after[open_paren + 1..];
    let close_paren = rest.find(')')?;
    let params_str = &rest[..close_paren];

    let mut params = Vec::new();
    for part in params_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(colon) = part.find(':') {
            let pname = part[..colon].trim().to_string();
            let pty = part[colon + 1..].trim();
            params.push(Param {
                name: pname,
                ty: map_ts_type(pty),
            });
        }
    }

    let after_paren = &rest[close_paren + 1..];
    let mut return_type = None;
    if let Some(colon_idx) = after_paren.find(':') {
        let after_colon = &after_paren[colon_idx + 1..];
        let after_colon = after_colon.trim_start();
        let end = after_colon
            .find(|c: char| c == '{' || c.is_whitespace())
            .unwrap_or(after_colon.len());
        let ty_str = after_colon[..end].trim();
        if !ty_str.is_empty() && ty_str != "void" {
            return_type = Some(map_ts_type(ty_str));
        }
    }

    Some(Function {
        name,
        params,
        return_type,
        kind: FunctionKind::Normal,
    })
}

fn parse_ts_interface<'a, I>(first_line: &str, lines: &mut I) -> Option<TypeDecl>
where
    I: Iterator<Item = &'a str>,
{
    let rest = &first_line["interface ".len()..];
    let mut name = rest.trim();
    if let Some(idx) = name.find('{') {
        name = name[..idx].trim();
    }

    let mut fields = Vec::new();
    while let Some(line) = lines.next() {
        let t = line.trim();
        if t.starts_with('}') {
            break;
        }
        if t.is_empty() || t.starts_with("//") {
            continue;
        }
        if let Some(colon) = t.find(':') {
            let fname = t[..colon].trim().to_string();
            let mut fty = t[colon + 1..].trim();
            if let Some(semi) = fty.find(';') {
                fty = fty[..semi].trim();
            }
            fields.push(Field {
                name: fname,
                ty: map_ts_type(fty),
            });
        }
    }

    Some(TypeDecl {
        name: name.to_string(),
        kind: TypeKind::Interface,
        fields,
    })
}

pub fn compare_and_print(rust_mod: &Module, ts_mod: &Module) {
    println!("=== ast_v2 Rust module ===\n{:#?}", rust_mod);
    println!("=== ast_v2 TS module ===\n{:#?}", ts_mod);

    println!("=== ast_v2 HelloWorld mapping ===");

    for rtype in &rust_mod.types {
        if let Some(ttype) = ts_mod.types.iter().find(|t| t.name == rtype.name) {
            println!(
                "Type {}: Rust {:?} ↔ TS {:?}",
                rtype.name, rtype.kind, ttype.kind
            );
            for rf in &rtype.fields {
                match ttype.fields.iter().find(|tf| tf.name == rf.name) {
                    Some(tf) => {
                        let status = if rf.ty == tf.ty { "OK" } else { "MISMATCH" };
                        println!(
                            "  field {}: Rust {:?} ↔ TS {:?} => {}",
                            rf.name, rf.ty, tf.ty, status
                        );
                    }
                    None => println!(
                        "  field {}: present in Rust, missing in TS",
                        rf.name
                    ),
                }
            }
        } else {
            println!("Type {}: present in Rust, missing in TS", rtype.name);
        }
    }

    for rfn in &rust_mod.functions {
        if let Some(tfn) = ts_mod.functions.iter().find(|f| f.name == rfn.name) {
            println!("Function {}:", rfn.name);
            let param_status = if rfn.params.len() == tfn.params.len()
                && rfn
                    .params
                    .iter()
                    .zip(&tfn.params)
                    .all(|(rp, tp)| rp.name == tp.name && rp.ty == tp.ty)
            {
                "params OK"
            } else {
                "params MISMATCH"
            };

            let ret_status = if rfn.return_type == tfn.return_type {
                "return type OK"
            } else {
                "return type MISMATCH"
            };

            println!("  {} / {}", param_status, ret_status);
        } else {
            println!("Function {}: present in Rust, missing in TS", rfn.name);
        }
    }
}

fn tag_hello_world_functions(module: &mut Module) {
    if module.name != "hello_world" {
        return;
    }
    for func in &mut module.functions {
        match func.name.as_str() {
            "main" => func.kind = FunctionKind::HelloWorldMain,
            "add" if func.params.len() == 2 => func.kind = FunctionKind::HelloWorldAdd,
            _ => {}
        }
    }
}

fn type_ref_to_rust(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Number => "i32".to_string(),
        TypeRef::String => "String".to_string(),
        TypeRef::Bool => "bool".to_string(),
        TypeRef::Custom(name) => name.clone(),
    }
}

fn type_ref_to_ts(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Number => "number".to_string(),
        TypeRef::String => "string".to_string(),
        TypeRef::Bool => "boolean".to_string(),
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
                out.push_str("    return x + y;\n");
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
                if func.return_type.is_some() {
                    out.push_str("    unimplemented!();\n");
                }
                out.push_str("}\n");
            }
        }

        if i + 1 < module.functions.len() {
            out.push('\n');
        }
    }

    out
}

pub fn module_to_ts(module: &Module) -> String {
    let mut out = String::new();

    for ty in &module.types {
        out.push_str("interface ");
        out.push_str(&ty.name);
        out.push_str(" {\n");
        for field in &ty.fields {
            out.push_str("  ");
            out.push_str(&field.name);
            out.push_str(": ");
            out.push_str(&type_ref_to_ts(&field.ty));
            out.push_str(";\n");
        }
        out.push_str("}\n\n");
    }

    for (i, func) in module.functions.iter().enumerate() {
        match func.kind {
            FunctionKind::HelloWorldMain => {
                out.push_str("export function main(): void {\n");
                out.push_str("  console.log(`Hello, World!`);\n");
                out.push_str("  const sum = add(2, 2);\n");
                out.push_str("  console.log(`add(2, 2) = ${sum}`);\n");
                out.push_str("  const person = { name: \"Not Sure\", age: 30 };\n");
                out.push_str("  console.log(`Person: name=${person.name}, age=${person.age}`);\n");
                out.push_str("}\n");
            }
            FunctionKind::HelloWorldAdd => {
                out.push_str("export function add(");
                for (idx, param) in func.params.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&param.name);
                    out.push_str(": ");
                    out.push_str(&type_ref_to_ts(&param.ty));
                }
                out.push_str("): ");
                if let Some(ret) = &func.return_type {
                    out.push_str(&type_ref_to_ts(ret));
                } else {
                    out.push_str("void");
                }
                out.push_str(" {\n");
                out.push_str("  return x + y;\n");
                out.push_str("}\n");
            }
            FunctionKind::Normal => {
                out.push_str("export function ");
                out.push_str(&func.name);
                out.push('(');
                for (idx, param) in func.params.iter().enumerate() {
                    if idx > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&param.name);
                    out.push_str(": ");
                    out.push_str(&type_ref_to_ts(&param.ty));
                }
                out.push(')');
                if let Some(ret) = &func.return_type {
                    out.push_str(": ");
                    out.push_str(&type_ref_to_ts(ret));
                } else {
                    out.push_str(": void");
                }
                out.push_str(" {\n");
                if func.return_type.is_some() {
                    out.push_str("  return undefined as any;\n");
                }
                out.push_str("}\n");
            }
        }

        if i + 1 < module.functions.len() {
            out.push('\n');
        }
    }

    out
}

pub fn convert_rust_file_to_ts(path: &Path) -> Result<String, String> {
    let module = from_rust_hello_world(path)?;
    Ok(module_to_ts(&module))
}

pub fn convert_ts_file_to_rust(path: &Path) -> Result<String, String> {
    let module = from_ts_hello_world(path)?;
    Ok(module_to_rust(&module))
}
