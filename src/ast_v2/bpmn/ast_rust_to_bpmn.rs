use std::collections::HashMap;

use crate::ast_v2::ast::{Module, TypeKind, TypeRef};
use crate::ast_v2::rust::from_rust_src;

use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

use super::ast_bpmn::{
    emit_bpmn_xml, hex_encode, parse_bpmn_xml, BpmnNode, BpmnProcess, BpmnSequenceFlow,
    RustFieldSig, RustParamSig,
};

fn rust_source_to_bpmn_process(rust_src: &str, label: Option<String>) -> BpmnProcess {
    let start_id = "StartEvent_1".to_string();
    let node_id = "RustSource_1".to_string();
    let end_id = "EndEvent_1".to_string();

    let line_path = label.clone();

    let mut nodes: Vec<BpmnNode> = Vec::new();
    nodes.push(BpmnNode::StartEvent {
        id: start_id.clone(),
        name: Some("Start".to_string()),
    });
    nodes.push(BpmnNode::RustSource {
        id: node_id.clone(),
        path: label,
        content_hex: hex_encode(rust_src.as_bytes()),
    });

    // Emit a per-line representation of the full Rust source (including comments and blank lines).
    // This is the preferred source for BPMN -> Rust reconstruction.
    for (idx, (text, eol)) in split_rust_lines(rust_src).into_iter().enumerate() {
        let id = format!("RustLine_{}", idx + 1);
        nodes.push(BpmnNode::RustLine {
            id,
            path: line_path.clone(),
            line: idx + 1,
            eol,
            text: text.clone(),
            text_hex: hex_encode(text.as_bytes()),
        });
    }

    nodes.push(BpmnNode::EndEvent {
        id: end_id.clone(),
        name: Some("End".to_string()),
    });

    let flows = vec![
        BpmnSequenceFlow {
            id: "Flow_1".to_string(),
            source_ref: start_id,
            target_ref: node_id.clone(),
            name: None,
        },
        BpmnSequenceFlow {
            id: "Flow_2".to_string(),
            source_ref: node_id,
            target_ref: end_id,
            name: None,
        },
    ];

    let mut proc = BpmnProcess {
        id: "rust_source".to_string(),
        name: None,
        nodes,
        flows,
    };

    // Informational-only structured view: if we can parse `fn main() { ... }`, emit
    // additional BPMN nodes representing statements/loops/ifs.
    // Round-trip back to Rust still prefers the embedded `rustSource` payload.
    if let Ok(parsed) = syn::parse_file(rust_src) {
        if let Some(main) = parsed.items.iter().find_map(|it| match it {
            syn::Item::Fn(f) if f.sig.ident == "main" => Some(f),
            _ => None,
        }) {
            if let Err(_e) = append_structured_main(&mut proc, &main.block) {
                // Best-effort only; ignore errors.
            }
        }
    }

    proc
}

fn split_rust_lines(src: &str) -> Vec<(String, String)> {
    // Produces (line_text_without_newline, eol_kind) where eol_kind is: LF, CRLF, NONE.
    // Preserves empty lines and trailing spaces.
    let bytes = src.as_bytes();
    let mut out: Vec<(String, String)> = Vec::new();
    let mut line_start: usize = 0;
    let mut i: usize = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            if i > 0 && bytes[i - 1] == b'\r' {
                let line_end = i - 1;
                out.push((src[line_start..line_end].to_string(), "CRLF".to_string()));
            } else {
                let line_end = i;
                out.push((src[line_start..line_end].to_string(), "LF".to_string()));
            }
            line_start = i + 1;
        }
        i += 1;
    }

    if line_start < bytes.len() {
        out.push((src[line_start..].to_string(), "NONE".to_string()));
    }

    out
}

#[derive(Default)]
struct BpmnBuild {
    next_node: usize,
    next_flow: usize,
}

impl BpmnBuild {
    fn new() -> Self {
        Self {
            next_node: 1,
            next_flow: 1,
        }
    }

    fn fresh_node_id(&mut self, prefix: &str) -> String {
        let id = format!("{}_{}", prefix, self.next_node);
        self.next_node += 1;
        id
    }

    fn fresh_flow_id(&mut self) -> String {
        let id = format!("Flow_S_{}", self.next_flow);
        self.next_flow += 1;
        id
    }

    fn push_flow(&mut self, proc: &mut BpmnProcess, source: String, target: String, name: Option<String>) {
        proc.flows.push(BpmnSequenceFlow {
            id: self.fresh_flow_id(),
            source_ref: source,
            target_ref: target,
            name,
        });
    }

    fn push_task(&mut self, proc: &mut BpmnProcess, name: String) -> String {
        let id = self.fresh_node_id("Task");
        proc.nodes.push(BpmnNode::ServiceTask {
            id: id.clone(),
            name: Some(name),
        });
        id
    }

    fn push_gateway(&mut self, proc: &mut BpmnProcess, kind: &str, name: String) -> String {
        let id = self.fresh_node_id(kind);
        match kind {
            "ExclusiveGateway" => proc.nodes.push(BpmnNode::ExclusiveGateway {
                id: id.clone(),
                name: Some(name),
            }),
            "ParallelGateway" => proc.nodes.push(BpmnNode::ParallelGateway {
                id: id.clone(),
                name: Some(name),
            }),
            _ => proc.nodes.push(BpmnNode::ExclusiveGateway {
                id: id.clone(),
                name: Some(name),
            }),
        }
        id
    }
}

fn expr_to_pretty(expr: &syn::Expr) -> String {
    // Best-effort pretty printing without relying on spans.
    match expr {
        syn::Expr::Path(p) => p
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        syn::Expr::Lit(l) => match &l.lit {
            syn::Lit::Int(i) => i.base10_digits().to_string(),
            syn::Lit::Float(f) => f.base10_digits().to_string(),
            syn::Lit::Str(s) => format!("\"{}\"", s.value()),
            syn::Lit::Bool(b) => (if b.value { "true" } else { "false" }).to_string(),
            _ => "<lit>".to_string(),
        },
        syn::Expr::MethodCall(m) => {
            let recv = expr_to_pretty(&m.receiver);
            let method = m.method.to_string();
            let args = m
                .args
                .iter()
                .map(expr_to_pretty)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}.{}({})", recv, method, args)
        }
        syn::Expr::Call(c) => {
            let func = expr_to_pretty(&c.func);
            let args = c
                .args
                .iter()
                .map(expr_to_pretty)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", func, args)
        }
        syn::Expr::Range(r) => {
            let start = r.start.as_ref().map(|e| expr_to_pretty(e)).unwrap_or_default();
            let end = r.end.as_ref().map(|e| expr_to_pretty(e)).unwrap_or_default();
            format!("{}..{}", start, end)
        }
        syn::Expr::Unary(u) => format!("-{}", expr_to_pretty(&u.expr)),
        _ => "<expr>".to_string(),
    }
}

fn stmt_to_nodes(proc: &mut BpmnProcess, b: &mut BpmnBuild, prev: String, stmt: &syn::Stmt) -> String {
    match stmt {
        syn::Stmt::Local(local) => {
            let (pat, is_mut) = match &local.pat {
                syn::Pat::Ident(i) => (i.ident.to_string(), i.mutability.is_some()),
                _ => ("_".to_string(), false),
            };
            let rhs = local
                .init
                .as_ref()
                .map(|i| expr_to_pretty(&i.expr))
                .unwrap_or_else(|| "<init>".to_string());
            let name = if is_mut {
                format!("letmut: {} = {}", pat, rhs)
            } else {
                format!("let: {} = {}", pat, rhs)
            };
            let tid = b.push_task(proc, name);
            b.push_flow(proc, prev, tid.clone(), None);
            tid
        }
        syn::Stmt::Expr(expr, _) => expr_to_nodes(proc, b, prev, expr),
        syn::Stmt::Item(_) => {
            // Ignore nested items for now.
            prev
        }
        syn::Stmt::Macro(m) => {
            let path = m.mac.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_default();
            if path == "println" {
                if let Some(action) = extract_println_action(m.mac.tokens.clone()) {
                    let name = match action {
                        MainAction::Println(s) => format!("println: {}", s),
                        MainAction::PrintlnVar(v) => format!("println_var: {}", v),
                        _ => "println: <args>".to_string(),
                    };
                    let tid = b.push_task(proc, name);
                    b.push_flow(proc, prev, tid.clone(), None);
                    return tid;
                }
            }

            let tid = b.push_task(proc, format!("comment: macro {}(...)", path));
            b.push_flow(proc, prev, tid.clone(), None);
            tid
        }
    }
}

fn expr_to_nodes(proc: &mut BpmnProcess, b: &mut BpmnBuild, prev: String, expr: &syn::Expr) -> String {
    match expr {
        syn::Expr::ForLoop(f) => {
            let pat = match &*f.pat {
                syn::Pat::Ident(i) => i.ident.to_string(),
                _ => "_".to_string(),
            };
            let iter = expr_to_pretty(&f.expr);
            let gate = b.push_gateway(proc, "ExclusiveGateway", format!("for: {} in {}", pat, iter));
            b.push_flow(proc, prev, gate.clone(), None);

            // Branch: loop body
            let body_entry = b.push_task(proc, format!("comment: loop body {}", pat));
            b.push_flow(proc, gate.clone(), body_entry.clone(), Some("loop".to_string()));
            let mut cur = body_entry;
            for s in &f.body.stmts {
                cur = stmt_to_nodes(proc, b, cur, s);
            }
            // Back-edge to loop gate
            b.push_flow(proc, cur, gate.clone(), Some("next".to_string()));

            // Exit branch
            let after = b.push_task(proc, format!("comment: end for {}", pat));
            b.push_flow(proc, gate.clone(), after.clone(), Some("exit".to_string()));
            after
        }
        syn::Expr::If(i) => {
            fn cond_label(i: &syn::ExprIf) -> String {
                match &*i.cond {
                    syn::Expr::Let(l) => {
                        let pat = match &*l.pat {
                            syn::Pat::Ident(id) => id.ident.to_string(),
                            _ => "_".to_string(),
                        };
                        format!("iflet: {} = {}", pat, expr_to_pretty(&l.expr))
                    }
                    other => format!("if: {}", expr_to_pretty(other)),
                }
            }

            fn emit_if_chain(proc: &mut BpmnProcess, b: &mut BpmnBuild, prev: String, i: &syn::ExprIf) -> String {
                // Use an ExclusiveGateway as a merge so it shows as a diamond.
                let merge = b.push_gateway(proc, "ExclusiveGateway", "merge".to_string());

                fn emit_one(proc: &mut BpmnProcess, b: &mut BpmnBuild, from: String, incoming_name: Option<String>, i: &syn::ExprIf, merge: &String) {
                    let gate = b.push_gateway(proc, "ExclusiveGateway", cond_label(i));
                    b.push_flow(proc, from, gate.clone(), incoming_name);

                    // then branch ("yes" exits to the right in the editor)
                    let then_entry = b.push_task(proc, "comment: then".to_string());
                    b.push_flow(proc, gate.clone(), then_entry.clone(), Some("yes".to_string()));
                    let mut then_cur = then_entry;
                    for s in &i.then_branch.stmts {
                        then_cur = stmt_to_nodes(proc, b, then_cur, s);
                    }
                    b.push_flow(proc, then_cur, merge.clone(), None);

                    // else / else-if chain ("no" flows downward)
                    if let Some((_, else_expr)) = &i.else_branch {
                        if let syn::Expr::If(next_if) = &**else_expr {
                            emit_one(proc, b, gate.clone(), Some("no".to_string()), next_if, merge);
                        } else {
                            let else_entry = b.push_task(proc, "comment: else".to_string());
                            b.push_flow(proc, gate.clone(), else_entry.clone(), Some("no".to_string()));
                            let else_cur = expr_to_nodes(proc, b, else_entry, else_expr);
                            b.push_flow(proc, else_cur, merge.clone(), None);
                        }
                    } else {
                        // No else: just fall through to merge on "no".
                        b.push_flow(proc, gate.clone(), merge.clone(), Some("no".to_string()));
                    }
                }

                emit_one(proc, b, prev, None, i, &merge);
                merge
            }

            emit_if_chain(proc, b, prev, i)
        }
        syn::Expr::Call(_) | syn::Expr::MethodCall(_) => {
            let tid = b.push_task(proc, format!("call: {}", expr_to_pretty(expr)));
            b.push_flow(proc, prev, tid.clone(), None);
            tid
        }
        _ => {
            let tid = b.push_task(proc, format!("comment: expr {}", expr_to_pretty(expr)));
            b.push_flow(proc, prev, tid.clone(), None);
            tid
        }
    }
}

fn append_structured_main(proc: &mut BpmnProcess, block: &syn::Block) -> Result<(), String> {
    // Find the RustSource_1 and EndEvent_1; splice structured nodes between them.
    let rust_source_id = "RustSource_1".to_string();
    let end_id = "EndEvent_1".to_string();

    // Remove the existing RustSource -> End flow, and remember its flow id.
    let mut removed = false;
    proc.flows.retain(|f| {
        if f.source_ref == rust_source_id && f.target_ref == end_id {
            removed = true;
            false
        } else {
            true
        }
    });
    if !removed {
        // If the process shape isn't as expected, skip.
        return Ok(());
    }

    let mut b = BpmnBuild::new();
    let entry = b.push_task(proc, "comment: structured main()".to_string());
    b.push_flow(proc, rust_source_id.clone(), entry.clone(), None);

    let mut cur = entry;
    for s in &block.stmts {
        cur = stmt_to_nodes(proc, &mut b, cur, s);
    }

    // Connect last structured node to EndEvent_1.
    b.push_flow(proc, cur, end_id, None);
    Ok(())
}

fn should_embed_lossless_rust_source(rust_src: &str) -> bool {
    // Heuristic: NeuralNetwork is too complex for the AST-v2 stub emitter today.
    // Embedding keeps round-trips lossless.
    rust_src.contains("NeuralNetwork") || rust_src.len() > 4000
}

fn main_contains_control_flow(rust_src: &str) -> bool {
    fn expr_has_control_flow(expr: &syn::Expr) -> bool {
        match expr {
            syn::Expr::If(i) => {
                // includes else-if chains.
                if i.then_branch.stmts.iter().any(stmt_has_control_flow) {
                    return true;
                }
                if let Some((_, else_expr)) = &i.else_branch {
                    if expr_has_control_flow(else_expr) {
                        return true;
                    }
                }
                true
            }
            syn::Expr::ForLoop(f) => {
                if f.body.stmts.iter().any(stmt_has_control_flow) {
                    return true;
                }
                true
            }
            syn::Expr::Block(b) => b.block.stmts.iter().any(stmt_has_control_flow),
            syn::Expr::While(w) => w.body.stmts.iter().any(stmt_has_control_flow) || true,
            syn::Expr::Loop(l) => l.body.stmts.iter().any(stmt_has_control_flow) || true,
            syn::Expr::Match(_) => true,
            syn::Expr::Let(l) => expr_has_control_flow(&l.expr),
            syn::Expr::Call(c) => {
                if expr_has_control_flow(&c.func) {
                    return true;
                }
                c.args.iter().any(expr_has_control_flow)
            }
            syn::Expr::MethodCall(m) => {
                if expr_has_control_flow(&m.receiver) {
                    return true;
                }
                m.args.iter().any(expr_has_control_flow)
            }
            syn::Expr::Assign(a) => expr_has_control_flow(&a.left) || expr_has_control_flow(&a.right),
            syn::Expr::Binary(b) => expr_has_control_flow(&b.left) || expr_has_control_flow(&b.right),
            syn::Expr::Unary(u) => expr_has_control_flow(&u.expr),
            syn::Expr::Paren(p) => expr_has_control_flow(&p.expr),
            syn::Expr::Group(g) => expr_has_control_flow(&g.expr),
            syn::Expr::Reference(r) => expr_has_control_flow(&r.expr),
            syn::Expr::Index(i) => expr_has_control_flow(&i.expr) || expr_has_control_flow(&i.index),
            syn::Expr::Field(f) => expr_has_control_flow(&f.base),
            syn::Expr::Return(r) => r
                .expr
                .as_ref()
                .is_some_and(|e| expr_has_control_flow(e.as_ref())),
            _ => false,
        }
    }

    fn stmt_has_control_flow(stmt: &syn::Stmt) -> bool {
        match stmt {
            syn::Stmt::Expr(e, _) => expr_has_control_flow(e),
            syn::Stmt::Local(l) => l
                .init
                .as_ref()
                .is_some_and(|i| expr_has_control_flow(&i.expr)),
            syn::Stmt::Macro(_) | syn::Stmt::Item(_) => false,
        }
    }

    let Ok(parsed) = syn::parse_file(rust_src) else {
        return false;
    };
    let Some(main) = parsed.items.iter().find_map(|it| match it {
        syn::Item::Fn(f) if f.sig.ident == "main" => Some(f),
        _ => None,
    }) else {
        return false;
    };

    main.block.stmts.iter().any(stmt_has_control_flow)
}

fn collect_rust_files_recursively(root: &Path) -> Result<Vec<PathBuf>, String> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
        for entry in fs::read_dir(dir).map_err(|e| format!("Failed to read dir {}: {}", dir.display(), e))? {
            let entry = entry.map_err(|e| format!("Failed to read dir entry in {}: {}", dir.display(), e))?;
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                // Skip common build dirs
                if name.eq_ignore_ascii_case("target") || name.eq_ignore_ascii_case("node_modules") {
                    continue;
                }
                walk(&path, out)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path);
            }
        }
        Ok(())
    }

    let mut out: Vec<PathBuf> = Vec::new();
    if root.is_dir() {
        walk(root, &mut out)?;
    } else {
        out.push(root.to_path_buf());
    }
    out.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    Ok(out)
}

pub fn convert_rust_dir_to_bpmn_xml(root: &Path) -> Result<String, String> {
    let files = collect_rust_files_recursively(root)?;
    if files.is_empty() {
        return Err(format!("No .rs files found under {}", root.display()));
    }

    let start_id = "StartEvent_1".to_string();
    let end_id = "EndEvent_1".to_string();

    let mut nodes: Vec<BpmnNode> = Vec::new();
    nodes.push(BpmnNode::StartEvent {
        id: start_id.clone(),
        name: Some("Start".to_string()),
    });

    let mut flows: Vec<BpmnSequenceFlow> = Vec::new();
    let mut prev = start_id.clone();
    let mut idx = 1usize;

    for (i, path) in files.iter().enumerate() {
        let src = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let rel = path.strip_prefix(root).ok().map(|p| p.to_string_lossy().to_string());
        let node_id = format!("RustSource_{}", i + 1);
        nodes.push(BpmnNode::RustSource {
            id: node_id.clone(),
            path: rel,
            content_hex: hex_encode(src.as_bytes()),
        });

        flows.push(BpmnSequenceFlow {
            id: format!("Flow_{}", idx),
            source_ref: prev,
            target_ref: node_id.clone(),
            name: None,
        });
        idx += 1;
        prev = node_id;
    }

    nodes.push(BpmnNode::EndEvent {
        id: end_id.clone(),
        name: Some("End".to_string()),
    });
    flows.push(BpmnSequenceFlow {
        id: format!("Flow_{}", idx),
        source_ref: prev,
        target_ref: end_id,
        name: None,
    });

    Ok(emit_bpmn_xml(&BpmnProcess {
        id: "rust_dir".to_string(),
        name: Some(root.file_name().and_then(|s| s.to_str()).unwrap_or("rust_dir").to_string()),
        nodes,
        flows,
    }))
}

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

pub fn convert_rust_code_to_bpmn_xml_with_path(
    rust_src: &str,
    path: Option<String>,
) -> Result<String, String> {
    // If the input has control flow (if/elseif/for/loop/match), use the rustSource+structured
    // representation so gateways show up as diamonds in BPMN editors.
    if should_embed_lossless_rust_source(rust_src) || main_contains_control_flow(rust_src) {
        return Ok(emit_bpmn_xml(&rust_source_to_bpmn_process(rust_src, path)));
    }

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

pub fn convert_rust_code_to_bpmn_xml(rust_src: &str) -> Result<String, String> {
    convert_rust_code_to_bpmn_xml_with_path(rust_src, None)
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
