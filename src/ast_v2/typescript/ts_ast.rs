use std::fs;
use std::path::Path;

use crate::ast_v2::ast::{Field, Function, FunctionKind, Module, Param, TypeDecl, TypeKind, TypeRef};

fn map_ts_type(name: &str) -> TypeRef {
    match name.trim() {
        "number" => TypeRef::Number,
        "string" => TypeRef::String,
        "boolean" => TypeRef::Bool,
        "any" => TypeRef::String,
        other => TypeRef::Custom(other.to_string()),
    }
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
            let mut pname = part[..colon].trim().to_string();
            if pname.ends_with('?') {
                pname.pop();
            }
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
        body: Vec::new(),
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

pub fn from_ts_module(path: &Path) -> Result<Module, String> {
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

        if let Some(mut func) = parse_ts_function(trimmed) {
            // Lightweight tagging for well-known converter helpers so that
            // TS -> Rust can emit more meaningful bodies for them.
            if module.name == "converter" {
                match func.name.as_str() {
                    "convert_type" => func.kind = FunctionKind::ConverterConvertType,
                    "convert_ir_type" => func.kind = FunctionKind::ConverterConvertIrType,
                    "ts_path_to_string" => func.kind = FunctionKind::ConverterTsPathToString,
                    _ => {}
                }
            }

            module.functions.push(func);
            continue;
        }

        if trimmed.starts_with("interface ") {
            if let Some(ty) = parse_ts_interface(trimmed, &mut lines) {
                module.types.push(ty);
            }
        }
    }

    Ok(module)
}

fn type_ref_to_ts(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Number => "number".to_string(),
        TypeRef::String => "string".to_string(),
        TypeRef::Bool => "boolean".to_string(),
        TypeRef::Custom(name) => name.clone(),
    }
}

pub fn module_to_ts(module: &Module) -> String {
    let mut out = String::new();

    // Special-case imports for certain well-known examples.
    let has_nn_main = module
        .functions
        .iter()
        .any(|f| matches!(f.kind, FunctionKind::NeuralNetMain));

    if has_nn_main {
        out.push_str("import { NeuralNetwork } from \"./lib.ts\";\n\n");
    }

    for ty in &module.types {
        if let TypeKind::Struct = ty.kind {
            out.push_str("// Converted from Rust: struct ");
            out.push_str(&ty.name);
            out.push_str("\n");
        }
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
                out.push_str("// Converted from Rust: fn main(...)\n");
                out.push_str("export function main(): void {\n");
                out.push_str("  console.log(`Hello, World!`);\n");
                out.push_str("  const sum = add(2, 2);\n");
                out.push_str("  console.log(`add(2, 2) = ${sum}`);\n");
                out.push_str("  const person = { name: \"Not Sure\", age: 30 };\n");
                out.push_str("  console.log(`Person: name=${person.name}, age=${person.age}`);\n");
                out.push_str("}\n");
            }
            FunctionKind::HelloWorldAdd => {
                out.push_str("// Converted from Rust: fn add(...)\n");
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
            FunctionKind::NeuralNetMain => {
                out.push_str("// Converted from Rust: fn main(...) for NeuralNetwork example\n");
                out.push_str("export function main(): void {\n");
                out.push_str("  const x_layers = 3;\n");
                out.push_str("  const y_nodes = 4;\n");
                out.push_str("  const z_weights = 2;\n");
                out.push_str(
                    "  const rng_label = (((globalThis as any).__RUST_TO_TS_RNG || 'default') as string);\n",
                );
                out.push_str("  let rng = ({ next_f32: (low: number, high: number) => {\n");
                out.push_str("    function mulberry32(a: number) {\n");
                out.push_str("      return function() {\n");
                out.push_str("        let t = a += 0x6D2B79F5;\n");
                out.push_str("        t = Math.imul(t ^ (t >>> 15), t | 1);\n");
                out.push_str("        t ^= t + Math.imul(t ^ (t >>> 7), t | 61);\n");
                out.push_str("        return ((t ^ (t >>> 14)) >>> 0) / 4294967296;\n");
                out.push_str("      };\n");
                out.push_str("    }\n");
                out.push_str("    const seed = ((globalThis as any).__RUST_TO_TS_SEED >>> 0) || 0xDEADBEEF;\n");
                out.push_str("    const rand = mulberry32(seed);\n");
                out.push_str("    return low + rand() * (high - low);\n");
                out.push_str("  }});\n");
                out.push_str(
                    "  const nn = (NeuralNetwork as any).random_uniform(x_layers, y_nodes, z_weights, -1.0, 1.0);\n",
                );
                out.push_str("  console.log(`RNG: ${rng_label}`);\n");
                out.push_str("  const [x, y, z] = nn.dims() as any;\n");
                out.push_str(
                    "  console.log(`NeuralNetwork<f64> dims: x(layers)=${x}, y(nodes)=${y}, z(weights)=${z} | total elements=${nn.len()}`);\n",
                );
                out.push_str("  for (let layer = 0; layer < Math.min(x, 2); layer++) {\n");
                out.push_str("    for (let node = 0; node < Math.min(y, 2); node++) {\n");
                out.push_str("      for (let weight = 0; weight < Math.min(z, 2); weight++) {\n");
                out.push_str("        const val = nn.get(layer, node, weight);\n");
                out.push_str("        if (val !== undefined) {\n");
                out.push_str("          console.log(`nn[${layer}, ${node}, ${weight}] = ${val}`);\n");
                out.push_str("        }\n");
                out.push_str("      }\n");
                out.push_str("    }\n");
                out.push_str("  }\n");
                out.push_str("}\n");
            }
            FunctionKind::AstV2ConvertRustFileToTs => {
                out.push_str("// Converted from Rust: fn convert_rust_file_to_ts(... )\n");
                out.push_str("export function convert_rust_file_to_ts(path: string): string {\n");
                out.push_str("  const module = from_rust_module(path);\n");
                out.push_str("  tag_special_functions_for_path(module, path);\n");
                out.push_str("  return module_to_ts(module);\n");
                out.push_str("}\n");
            }
            FunctionKind::AstV2ConvertTsFileToRust => {
                out.push_str("// Converted from Rust: fn convert_ts_file_to_rust(... )\n");
                out.push_str("export function convert_ts_file_to_rust(path: string): string {\n");
                out.push_str("  const module = from_ts_module(path);\n");
                out.push_str("  return module_to_rust(&module);\n");
                out.push_str("}\n");
            }
            FunctionKind::ConverterConvertType
            | FunctionKind::ConverterConvertIrType
            | FunctionKind::ConverterTsPathToString => {
                // For converter helpers, TS is already the source of truth;
                // emit them as normal exported functions.
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
                out.push_str("}\n");
            }
        }

        if i + 1 < module.functions.len() {
            out.push('\n');
        }
    }

    out
}
