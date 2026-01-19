use std::collections::HashMap;

use crate::ast_v2::ast::{
    Field, Function, FunctionKind, Module, Param, TypeDecl, TypeKind, TypeRef,
};

use super::ast_bpmn::{
    best_effort_linearize, parse_bpmn_xml, BpmnNode, BpmnProcess, RustFieldSig, RustParamSig,
};

fn map_type_ref(name: &str) -> TypeRef {
    match name.trim() {
        "number" | "i32" | "i64" | "u32" | "u64" | "usize" | "isize" | "f32" | "f64" => {
            TypeRef::Number
        }
        "string" | "String" | "str" => TypeRef::String,
        "boolean" | "bool" => TypeRef::Bool,
        other => TypeRef::Custom(other.to_string()),
    }
}

fn map_type_kind(kind: &str) -> TypeKind {
    match kind.trim() {
        "interface" => TypeKind::Interface,
        _ => TypeKind::Struct,
    }
}

fn sanitize_ident(name: &str) -> String {
    let mut out = String::new();
    for (i, ch) in name.chars().enumerate() {
        let ok = ch.is_ascii_alphanumeric() || ch == '_';
        if ok {
            out.push(ch);
        } else if i == 0 {
            out.push('_');
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_task".to_string()
    } else {
        out
    }
}

fn unique_name(base: &str, used: &mut HashMap<String, usize>) -> String {
    let n = used.entry(base.to_string()).or_insert(0);
    *n += 1;
    if *n == 1 {
        base.to_string()
    } else {
        format!("{}_{}", base, *n)
    }
}

fn node_label(id: &str, name: &Option<String>) -> String {
    name.clone().unwrap_or_else(|| id.to_string())
}

fn rust_escape_string(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            other => out.push(other),
        }
    }
    out
}

fn rust_quote_string_literal(s: &str) -> String {
    format!("\"{}\"", rust_escape_string(s))
}

fn parse_ident(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let ok_first = s
        .chars()
        .next()
        .map(|ch| ch.is_ascii_alphabetic() || ch == '_')
        .unwrap_or(false);
    if !ok_first {
        return None;
    }
    if s.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
        Some(s.to_string())
    } else {
        None
    }
}

fn parse_simple_expr(expr: &str) -> Option<String> {
    // Supported subset for standard BPMN directives:
    // - numbers: 1, -1, 3.14
    // - bools: true/false
    // - strings: "..." (quotes required)
    // - identifiers: foo_bar
    let e = expr.trim();
    if e.is_empty() {
        return None;
    }

    if e.eq_ignore_ascii_case("true") {
        return Some("true".to_string());
    }
    if e.eq_ignore_ascii_case("false") {
        return Some("false".to_string());
    }

    if let Ok(_) = e.parse::<i64>() {
        return Some(e.to_string());
    }
    if let Ok(_) = e.parse::<f64>() {
        return Some(e.to_string());
    }

    if e.starts_with('"') && e.ends_with('"') && e.len() >= 2 {
        // Keep as a Rust string literal; re-escape to avoid leaking raw quotes.
        let inner = &e[1..e.len() - 1];
        return Some(rust_quote_string_literal(inner));
    }

    parse_ident(e)
}

fn parse_assignment_rhs(tail: &str) -> Option<(String, String)> {
    // "x = 1" or "x=1"
    let (lhs, rhs) = tail.split_once('=')?;
    let lhs = parse_ident(lhs)?;
    let rhs = parse_simple_expr(rhs)?;
    Some((lhs, rhs))
}

fn parse_call_target(name: &str) -> Option<String> {
    let trimmed = name.trim();
    let (head, tail) = trimmed.split_once(':')?;
    if head.trim().eq_ignore_ascii_case("call") {
        parse_ident(tail)
    } else {
        None
    }
}

fn parse_task_directive(name: &str) -> Option<Vec<String>> {
    // Convention for standard BPMN tasks:
    // - "println: <text>" => emits `println!("<text>");`
    // - "console.log: <text>" => alias for println
    // - "println_var: <ident>" => emits `println!("{}", <ident>);`
    // - "let: x = <expr>" => emits `let x = <expr>;`
    // - "letmut: x = <expr>" => emits `let mut x = <expr>;`
    // - "set: x = <expr>" => emits `x = <expr>;`
    // - "call: foo" => emits `foo();`
    // - "comment: ..." => emits `// ...`
    let trimmed = name.trim();
    let (head, tail) = trimmed.split_once(':')?;
    let head = head.trim().to_ascii_lowercase();
    let tail = tail.trim();
    if tail.is_empty() {
        return None;
    }

    match head.as_str() {
        "println" | "print" | "console.log" | "log" => {
            // Treat tail as raw text.
            let s = rust_escape_string(tail);
            Some(vec![format!("println!(\"{}\");", s)])
        }
        "println_var" => {
            let ident = parse_ident(tail)?;
            Some(vec![format!("println!(\"{{}}\", {});", ident)])
        }
        "let" => {
            let (lhs, rhs) = parse_assignment_rhs(tail)?;
            Some(vec![format!("let {} = {};", lhs, rhs)])
        }
        "letmut" | "let_mut" | "let-mut" => {
            let (lhs, rhs) = parse_assignment_rhs(tail)?;
            Some(vec![format!("let mut {} = {};", lhs, rhs)])
        }
        "set" => {
            let (lhs, rhs) = parse_assignment_rhs(tail)?;
            Some(vec![format!("{} = {};", lhs, rhs)])
        }
        "call" => {
            let ident = parse_ident(tail)?;
            Some(vec![format!("{}();", ident)])
        }
        "comment" => Some(vec![format!("// {}", tail)]),
        _ => None,
    }
}

pub fn bpmn_process_to_module(proc: &BpmnProcess) -> Module {
    // Best-effort mapping to the shared AST:
    // - Each (service)task becomes a Rust function stub.
    // - The main() body becomes a linearized call-chain (calls task functions).
    // - Gateways/events not representable in the base AST are preserved as comments.
    let mut used_names: HashMap<String, usize> = HashMap::new();
    let mut node_to_fn: HashMap<String, String> = HashMap::new();
    let mut node_to_inline: HashMap<String, Vec<String>> = HashMap::new();

    let mut types: Vec<TypeDecl> = Vec::new();
    let mut functions: Vec<Function> = Vec::new();

    for n in &proc.nodes {
        if let BpmnNode::RustType {
            id: _,
            name,
            kind,
            fields,
        } = n
        {
            let mapped_fields: Vec<Field> = fields
                .iter()
                .map(|RustFieldSig { name, ty }| Field {
                    name: name.clone(),
                    ty: map_type_ref(ty),
                })
                .collect();
            types.push(TypeDecl {
                name: name.clone(),
                kind: map_type_kind(kind),
                fields: mapped_fields,
            });
            continue;
        }

        if let BpmnNode::RustFunction {
            id: _,
            name,
            params,
            return_type,
            body,
        } = n
        {
            let mapped_params: Vec<Param> = params
                .iter()
                .map(|RustParamSig { name, ty }| Param {
                    name: name.clone(),
                    ty: map_type_ref(ty),
                })
                .collect();
            functions.push(Function {
                name: name.clone(),
                params: mapped_params,
                return_type: return_type.as_ref().map(|s| map_type_ref(s)),
                kind: FunctionKind::Normal,
                body: body.clone(),
            });
            continue;
        }

        let (id, name) = match n {
            BpmnNode::Task { id, name } => (id, name),
            BpmnNode::ServiceTask { id, name } => (id, name),
            _ => continue,
        };

        let label = node_label(id, name);

        if let Some(name) = name.as_ref() {
            if let Some(lines) = parse_task_directive(name) {
                // If this is a `call:` directive, ensure the called function exists.
                if let Some(target) = parse_call_target(name) {
                    let exists = functions.iter().any(|f| f.name == target);
                    if !exists {
                        functions.push(Function {
                            name: target,
                            params: vec![],
                            return_type: None,
                            kind: FunctionKind::Normal,
                            body: vec!["// TODO: implement called function".to_string()],
                        });
                    }
                }
                node_to_inline.insert(id.clone(), lines);
                continue;
            }
        }

        let base = sanitize_ident(&label);
        let fn_name = unique_name(&base, &mut used_names);

        node_to_fn.insert(id.clone(), fn_name.clone());
        functions.push(Function {
            name: fn_name,
            params: vec![],
            return_type: None,
            kind: FunctionKind::Normal,
            body: vec![format!("// TODO: implement BPMN task \"{}\"", label)],
        });
    }

    // main() best-effort linear order.
    let already_has_main = functions.iter().any(|f| f.name == "main");
    if !already_has_main {
        let mut main_body: Vec<String> = Vec::new();
        let order = best_effort_linearize(proc);
        if order.is_empty() {
            main_body.push(
                "// TODO: no linear path found (missing startEvent/sequenceFlow?)".to_string(),
            );
        } else {
            for node_id in order {
                if let Some(lines) = node_to_inline.get(&node_id) {
                    main_body.extend(lines.iter().cloned());
                } else if let Some(f) = node_to_fn.get(&node_id) {
                    main_body.push(format!("{}();", f));
                } else {
                    main_body.push(format!("// Skipping BPMN node id=\"{}\" (not a task)", node_id));
                }
            }
        }

        functions.push(Function {
            name: "main".to_string(),
            params: vec![],
            return_type: None,
            kind: FunctionKind::Normal,
            body: main_body,
        });
    }

    Module {
        name: proc.name.clone().unwrap_or_else(|| proc.id.clone()),
        types,
        functions,
    }
}

pub fn convert_bpmn_xml_to_module(xml: &str) -> Result<Module, String> {
    let proc = parse_bpmn_xml(xml)?;
    Ok(bpmn_process_to_module(&proc))
}

pub fn convert_bpmn_xml_to_rust_code(xml: &str) -> Result<String, String> {
    let module = convert_bpmn_xml_to_module(xml)?;
    Ok(crate::ast_v2::module_to_rust(&module))
}

#[cfg(test)]
mod tests {
    use crate::ast_v2::ast::{
        Field, Function, FunctionKind, Module, Param, TypeDecl, TypeKind, TypeRef,
    };
    use crate::ast_v2::bpmn::{convert_bpmn_xml_to_module, convert_module_to_bpmn_xml};

    #[test]
    fn module_round_trips_through_bpmn_xml() {
        let module = Module {
            name: "hello".to_string(),
            types: vec![TypeDecl {
                name: "Person".to_string(),
                kind: TypeKind::Struct,
                fields: vec![
                    Field {
                        name: "name".to_string(),
                        ty: TypeRef::String,
                    },
                    Field {
                        name: "age".to_string(),
                        ty: TypeRef::Number,
                    },
                ],
            }],
            functions: vec![
                Function {
                    name: "add".to_string(),
                    params: vec![
                        Param {
                            name: "x".to_string(),
                            ty: TypeRef::Number,
                        },
                        Param {
                            name: "y".to_string(),
                            ty: TypeRef::Number,
                        },
                    ],
                    return_type: Some(TypeRef::Number),
                    kind: FunctionKind::Normal,
                    body: vec!["return x + y;".to_string()],
                },
                Function {
                    name: "main".to_string(),
                    params: vec![],
                    return_type: None,
                    kind: FunctionKind::Normal,
                    body: vec!["let _ = add(2, 2);".to_string()],
                },
            ],
        };

        let xml = convert_module_to_bpmn_xml(&module).expect("module->bpmn xml");
        let module2 = convert_bpmn_xml_to_module(&xml).expect("bpmn xml->module");

        assert_eq!(module2.name, module.name);
        assert_eq!(module2.types, module.types);
        assert_eq!(module2.functions, module.functions);
    }
}
