use std::collections::HashMap;

use crate::ast_v2::ast::{Module, TypeKind, TypeRef};
use crate::ast_v2::rust::from_rust_src;

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use super::ast_bpmn::{
    emit_bpmn_xml, parse_bpmn_xml, BpmnNode, BpmnProcess, BpmnSequenceFlow, RustFieldSig,
    RustParamSig,
};

fn xml_name_from_ident(ident: &str) -> String {
    // BPMN name can be any string; keep ident.
    ident.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum MainAction {
    Println(String),
    PrintlnVar(String),
    Let { name: String, expr: String, mutable: bool },
    Set { name: String, expr: String },
    Call(String),
}

fn rust_escape_for_directive(s: &str) -> String {
    // Keep directives readable and reversible.
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

fn expr_to_simple_directive(expr: &syn::Expr) -> Option<String> {
    match expr {
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Int(i) => Some(i.base10_digits().to_string()),
            syn::Lit::Float(f) => Some(f.base10_digits().to_string()),
            syn::Lit::Bool(b) => Some(if b.value { "true" } else { "false" }.to_string()),
            syn::Lit::Str(s) => Some(format!("\"{}\"", rust_escape_for_directive(&s.value()))),
            _ => None,
        },
        syn::Expr::Unary(u) => {
            // Support negative numeric literals.
            if let syn::UnOp::Neg(_) = u.op {
                if let syn::Expr::Lit(l) = &*u.expr {
                    if let syn::Lit::Int(i) = &l.lit {
                        return Some(format!("-{}", i.base10_digits()));
                    }
                    if let syn::Lit::Float(f) = &l.lit {
                        return Some(format!("-{}", f.base10_digits()));
                    }
                }
            }
            None
        }
        syn::Expr::Path(p) => {
            if p.path.segments.len() == 1 {
                Some(p.path.segments[0].ident.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

struct ExprList {
    elems: Punctuated<syn::Expr, syn::Token![,]>,
}

impl Parse for ExprList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let elems = Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated(input)?;
        Ok(ExprList { elems })
    }
}

fn parse_macro_args(tokens: TokenStream) -> Option<Vec<syn::Expr>> {
    let list: ExprList = syn::parse2(tokens).ok()?;
    Some(list.elems.into_iter().collect())
}

fn extract_println_action(tokens: TokenStream) -> Option<MainAction> {
    // Support:
    // - println!("...")
    // - println!("{}", x)
    if let Ok(lit) = syn::parse2::<syn::LitStr>(tokens.clone()) {
        return Some(MainAction::Println(lit.value()));
    }

    let args = parse_macro_args(tokens)?;
    if args.len() == 2 {
        if let syn::Expr::Lit(first) = &args[0] {
            if let syn::Lit::Str(fmt) = &first.lit {
                if fmt.value() == "{}" {
                    if let syn::Expr::Path(p) = &args[1] {
                        if p.path.segments.len() == 1 {
                            return Some(MainAction::PrintlnVar(
                                p.path.segments[0].ident.to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    None
}

fn extract_main_actions(file: &syn::File) -> Vec<MainAction> {
    let main_fn = file.items.iter().find_map(|it| match it {
        syn::Item::Fn(f) if f.sig.ident == "main" => Some(f),
        _ => None,
    });

    let Some(main_fn) = main_fn else {
        return Vec::new();
    };

    let mut actions = Vec::new();
    for stmt in &main_fn.block.stmts {
        match stmt {
            syn::Stmt::Local(local) => {
                let syn::Pat::Ident(pat_ident) = &local.pat else {
                    continue;
                };
                let Some(init) = &local.init else {
                    continue;
                };
                let Some(expr) = expr_to_simple_directive(&init.expr) else {
                    continue;
                };
                actions.push(MainAction::Let {
                    name: pat_ident.ident.to_string(),
                    expr,
                    mutable: pat_ident.mutability.is_some(),
                });
            }
            syn::Stmt::Macro(m) => {
                if m.mac.path.is_ident("println") {
                    if let Some(action) = extract_println_action(m.mac.tokens.clone()) {
                        actions.push(action);
                    }
                }
            }
            syn::Stmt::Expr(expr, _) => match expr {
                syn::Expr::Assign(assign) => {
                    if let syn::Expr::Path(p) = &*assign.left {
                        if p.path.segments.len() == 1 {
                            let Some(expr) = expr_to_simple_directive(&assign.right) else {
                                continue;
                            };
                            actions.push(MainAction::Set {
                                name: p.path.segments[0].ident.to_string(),
                                expr,
                            });
                        }
                    }
                }
                syn::Expr::Call(call) => {
                    if let syn::Expr::Path(p) = &*call.func {
                        if p.path.segments.len() == 1 {
                            let ident = p.path.segments[0].ident.to_string();
                            if ident != "println" {
                                actions.push(MainAction::Call(ident));
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    actions
}

fn sanitize_id_fragment(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            out.push(ch);
        } else if i == 0 {
            out.push('_');
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "_".to_string()
    } else {
        out
    }
}

fn unique_id(base: &str, used: &mut HashMap<String, usize>) -> String {
    let n = used.entry(base.to_string()).or_insert(0);
    *n += 1;
    if *n == 1 {
        base.to_string()
    } else {
        format!("{}_{}", base, *n)
    }
}

fn type_ref_to_name(ty: &TypeRef) -> String {
    match ty {
        TypeRef::Number => "number".to_string(),
        TypeRef::String => "string".to_string(),
        TypeRef::Bool => "boolean".to_string(),
        TypeRef::Custom(s) => s.clone(),
    }
}

fn type_kind_to_name(kind: &TypeKind) -> String {
    match kind {
        TypeKind::Struct => "struct".to_string(),
        TypeKind::Interface => "interface".to_string(),
    }
}

pub fn module_to_bpmn_process(module: &Module) -> BpmnProcess {
    // Deterministic, lossless representation of the AST-v2 module using
    // non-standard BPMN nodes (`rustType`, `rustFunction`).
    // A simple linear flow is added across functions (Start -> fn1 -> ... -> End)
    // so that `best_effort_linearize` has something to traverse.

    let mut used_ids: HashMap<String, usize> = HashMap::new();

    let start_id = "StartEvent_1".to_string();
    let end_id = "EndEvent_1".to_string();

    let mut nodes: Vec<BpmnNode> = Vec::new();
    nodes.push(BpmnNode::StartEvent {
        id: start_id.clone(),
        name: Some("Start".to_string()),
    });

    for ty in &module.types {
        let base = format!("RustType_{}", sanitize_id_fragment(&ty.name));
        let id = unique_id(&base, &mut used_ids);
        let fields = ty
            .fields
            .iter()
            .map(|f| RustFieldSig {
                name: f.name.clone(),
                ty: type_ref_to_name(&f.ty),
            })
            .collect();
        nodes.push(BpmnNode::RustType {
            id,
            name: ty.name.clone(),
            kind: type_kind_to_name(&ty.kind),
            fields,
        });
    }

    let mut fn_node_ids: Vec<String> = Vec::new();
    for func in &module.functions {
        let base = format!("RustFunction_{}", sanitize_id_fragment(&func.name));
        let id = unique_id(&base, &mut used_ids);
        fn_node_ids.push(id.clone());
        let params = func
            .params
            .iter()
            .map(|p| RustParamSig {
                name: p.name.clone(),
                ty: type_ref_to_name(&p.ty),
            })
            .collect();
        let return_type = func.return_type.as_ref().map(type_ref_to_name);
        nodes.push(BpmnNode::RustFunction {
            id,
            name: func.name.clone(),
            params,
            return_type,
            body: func.body.clone(),
        });
    }

    nodes.push(BpmnNode::EndEvent {
        id: end_id.clone(),
        name: Some("End".to_string()),
    });

    let mut flows: Vec<BpmnSequenceFlow> = Vec::new();
    let mut prev = start_id.clone();
    let mut flow_idx = 1usize;
    for tid in &fn_node_ids {
        flows.push(BpmnSequenceFlow {
            id: format!("Flow_{}", flow_idx),
            source_ref: prev,
            target_ref: tid.clone(),
            name: None,
        });
        flow_idx += 1;
        prev = tid.clone();
    }
    flows.push(BpmnSequenceFlow {
        id: format!("Flow_{}", flow_idx),
        source_ref: prev,
        target_ref: end_id.clone(),
        name: None,
    });

    BpmnProcess {
        id: format!("module_{}", sanitize_id_fragment(&module.name)),
        name: Some(module.name.clone()),
        nodes,
        flows,
    }
}

pub fn rust_code_to_bpmn_process(rust_src: &str) -> Result<BpmnProcess, String> {
    let file: syn::File = syn::parse_file(rust_src).map_err(|e| format!("Rust parse error: {e}"))?;

    let fn_names: Vec<String> = file
        .items
        .iter()
        .filter_map(|it| match it {
            syn::Item::Fn(f) => Some(f.sig.ident.to_string()),
            _ => None,
        })
        .collect();

    // Prefer the main() statement order; fallback to all functions except main.
    let mut ordered_actions = extract_main_actions(&file);
    if ordered_actions.is_empty() {
        ordered_actions = fn_names
            .iter()
            .filter(|n| n.as_str() != "main")
            .cloned()
            .map(MainAction::Call)
            .collect();
    }

    let start_id = "StartEvent_1".to_string();
    let end_id = "EndEvent_1".to_string();

    let mut nodes: Vec<BpmnNode> = Vec::new();
    nodes.push(BpmnNode::StartEvent {
        id: start_id.clone(),
        name: Some("Start".to_string()),
    });

    let mut task_ids: Vec<String> = Vec::new();
    for (i, action) in ordered_actions.iter().enumerate() {
        let id = format!("ServiceTask_{}", i + 1);
        task_ids.push(id.clone());
        let name = match action {
            MainAction::Println(s) => Some(format!("println: {}", s)),
            MainAction::PrintlnVar(v) => Some(format!("println_var: {}", v)),
            MainAction::Let { name, expr, mutable } => {
                if *mutable {
                    Some(format!("letmut: {} = {}", name, expr))
                } else {
                    Some(format!("let: {} = {}", name, expr))
                }
            }
            MainAction::Set { name, expr } => Some(format!("set: {} = {}", name, expr)),
            MainAction::Call(f) => Some(format!("call: {}", xml_name_from_ident(f))),
        };
        nodes.push(BpmnNode::ServiceTask {
            id,
            name,
        });
    }

    nodes.push(BpmnNode::EndEvent {
        id: end_id.clone(),
        name: Some("End".to_string()),
    });

    let mut flows: Vec<BpmnSequenceFlow> = Vec::new();
    let mut prev = start_id.clone();
    let mut flow_idx = 1usize;
    for tid in &task_ids {
        flows.push(BpmnSequenceFlow {
            id: format!("Flow_{}", flow_idx),
            source_ref: prev,
            target_ref: tid.clone(),
            name: None,
        });
        flow_idx += 1;
        prev = tid.clone();
    }
    flows.push(BpmnSequenceFlow {
        id: format!("Flow_{}", flow_idx),
        source_ref: prev,
        target_ref: end_id.clone(),
        name: None,
    });

    Ok(BpmnProcess {
        id: "rust_process".to_string(),
        name: None,
        nodes,
        flows,
    })
}

pub fn convert_rust_code_to_bpmn_xml(rust_src: &str) -> Result<String, String> {
    // Prefer standard BPMN (`serviceTask` nodes) when we can extract a
    // meaningful action sequence from `main()` (e.g., println directives).
    // Otherwise fall back to a lossless AST-v2 representation (`rustFunction`).
    if let Ok(proc) = rust_code_to_bpmn_process(rust_src) {
        let has_standard_directive = proc.nodes.iter().any(|n| match n {
            BpmnNode::ServiceTask { name: Some(n), .. }
            | BpmnNode::Task { name: Some(n), .. } => {
                let low = n.trim_start().to_ascii_lowercase();
                low.starts_with("println:")
                    || low.starts_with("println_var:")
                    || low.starts_with("let:")
                    || low.starts_with("letmut:")
                    || low.starts_with("set:")
                    || low.starts_with("call:")
                    || low.starts_with("comment:")
            }
            _ => false,
        });
        if has_standard_directive {
            return Ok(emit_bpmn_xml(&proc));
        }
    }

    match from_rust_src(rust_src, "rust_process") {
        Ok(module) => Ok(emit_bpmn_xml(&module_to_bpmn_process(&module))),
        Err(_) => Ok(emit_bpmn_xml(&rust_code_to_bpmn_process(rust_src)?)),
    }
}

pub fn convert_module_to_bpmn_xml(_module: &Module) -> Result<String, String> {
    let proc = module_to_bpmn_process(_module);
    Ok(emit_bpmn_xml(&proc))
}

#[allow(dead_code)]
pub fn validate_bpmn_xml(xml: &str) -> Result<(), String> {
    let _ = parse_bpmn_xml(xml)?;
    Ok(())
}
