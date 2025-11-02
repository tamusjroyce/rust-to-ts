use syn::{File, Item, ItemFn, FnArg, ReturnType, Type, ItemStruct, Fields, Stmt, Expr, Block};
use syn::{ImplItem};
use syn::{Pat, PatTuple};
use std::fs;
use std::path::{Path, PathBuf};

pub fn convert_rust_to_ts(ast: &File, _source: &str) -> String {
    let mut output = String::new();
    
    for item in &ast.items {
        match item {
            Item::Fn(func) => {
                output.push_str(&convert_function(func));
            }
            Item::Struct(struct_item) => {
                output.push_str(&convert_struct(struct_item));
            }
            Item::Impl(impl_item) => {
                // Convert inherent impl methods to free functions prefixed with type name
                if impl_item.trait_.is_none() {
                    output.push_str(&convert_impl_inherent(impl_item));
                } else {
                    // Trait impls are skipped for now
                    output.push_str("// Skipped trait implementation (not yet supported)\n\n");
                }
            }
            Item::Use(_use_item) => {
                // External imports have no direct TS equivalent here; ignore silently
            }
            Item::Mod(_m) => {
                // Skip modules (e.g., #[cfg(test)] mod tests)
            }
            _ => {
                output.push_str("// Unsupported Rust item\n\n");
            }
        }
    }
    
    output
}

fn convert_function(func: &ItemFn) -> String {
    let mut output = String::new();
    
    // Add comment showing original Rust function
    output.push_str(&format!("// Converted from Rust: fn {}(...)\n", func.sig.ident));
    
    // Function name
    // Capture function generics <T, U, ...>
    let gen_params: Vec<String> = func
        .sig
        .generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect();
    let gen_suffix = if gen_params.is_empty() { String::new() } else { format!("<{}>", gen_params.join(", ")) };

    output.push_str(&format!("function {}{}(", func.sig.ident, gen_suffix));
    
    // Parameters
    let params: Vec<String> = func.sig.inputs.iter().map(|arg| {
        match arg {
            FnArg::Typed(pat_type) => {
                let param_name = match &*pat_type.pat {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => quote::quote!(#pat_type.pat).to_string(),
                };
                let param_type = convert_type(&pat_type.ty);
                format!("{}: {}", param_name, param_type)
            }
            _ => "this: any".to_string(),
        }
    }).collect();
    output.push_str(&params.join(", "));
    
    // Return type
    output.push_str(")");
    match &func.sig.output {
        ReturnType::Type(_, ty) => {
            output.push_str(&format!(": {}", convert_type(ty)));
        }
        ReturnType::Default => {
            output.push_str(": void");
        }
    }
    
    output.push_str(" {\n");
    
    // Convert function body
    output.push_str(&convert_block(&func.block));
    
    output.push_str("}\n\n");
    
    output
}

fn convert_block(block: &Block) -> String {
    let mut output = String::new();
    let last_idx = if block.stmts.is_empty() { None } else { Some(block.stmts.len() - 1) };

    for (i, stmt) in block.stmts.iter().enumerate() {
        // If the last statement is an expression without a semicolon, return it
        if let Some(last) = last_idx {
            if i == last {
            if let Stmt::Expr(expr, None) = stmt {
                let expr_str = convert_expr(expr);
                if expr_str.is_empty() || expr_str == "/* Unsupported expression */" {
                    output.push_str("  // Unsupported trailing expression\n");
                } else {
                    output.push_str("  return ");
                    output.push_str(&expr_str);
                    output.push_str(";\n");
                }
                continue;
            }
            }
        }

        output.push_str(&convert_stmt(stmt));
    }

    output
}

fn convert_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Macro(stmt_mac) => {
            // Handle statement macros like println! and print!
            let name = stmt_mac
                .mac
                .path
                .segments
                .last()
                .map(|s| s.ident.to_string())
                .unwrap_or_default();
            let args = stmt_mac.mac.tokens.to_string();
            match name.as_str() {
                "println" => {
                    let mut out = String::from("  // Rust macro\n");
                    out.push_str(&format!("  console.log({});\n", args.trim()));
                    out
                }
                "print" => {
                    let mut out = String::from("  // Rust macro\n");
                    out.push_str(&format!("  process.stdout.write({});\n", args.trim()));
                    out
                }
                _ => {
                    format!("  // Unsupported macro: {}!\\n", name)
                }
            }
        }
        Stmt::Local(local) => {
            let mut result = String::from("  // Rust variable declaration\n");
            // Determine variable names and whether it's a tuple destructure
            let (pat_ts, is_tuple, is_mut) = match &local.pat {
                Pat::Ident(p) => (p.ident.to_string(), false, p.mutability.is_some()),
                Pat::Tuple(PatTuple { elems, .. }) => {
                    let mut names: Vec<String> = Vec::new();
                    for (i, elem) in elems.iter().enumerate() {
                        match elem {
                            Pat::Ident(pi) => names.push(pi.ident.to_string()),
                            _ => names.push(format!("_tmp{}", i)),
                        }
                    }
                    (format!("[{}]", names.join(", ")), true, false)
                }
                _ => ("_tmp".to_string(), false, true),
            };

            let keyword = if is_mut { "let" } else { "const" };
            if let Some(init) = &local.init {
                let expr_str = convert_expr(&init.expr);
                if expr_str.contains("/* Unsupported expression */") {
                    // Avoid emitting broken assignment, just keep a comment
                    result.push_str("  // Unsupported initializer omitted\n");
                } else {
                    if is_tuple {
                        result.push_str(&format!("  {} {} = {} as any;\n", keyword, pat_ts, expr_str));
                    } else {
                        result.push_str(&format!("  {} {} = {};\n", keyword, pat_ts, expr_str));
                    }
                }
            } else {
                result.push_str(&format!("  {} {};\n", keyword, pat_ts));
            }
            result
        }
        Stmt::Expr(expr, semi) => {
            let expr_str = convert_expr(expr);
            if expr_str.is_empty() || expr_str == "/* Unsupported expression */" {
                return String::from("  // Unsupported statement\n");
            }
            let mut result = String::from("  // Rust expression\n");
            result.push_str(&format!("  {}", expr_str));
            if semi.is_some() {
                result.push(';');
            }
            result.push('\n');
            result
        }
        _ => String::from("  // Unsupported statement\n"),
    }
}

fn convert_expr(expr: &Expr) -> String {
    match expr {
        Expr::Lit(lit) => {
            match &lit.lit {
                syn::Lit::Int(i) => i.base10_digits().to_string(),
                syn::Lit::Float(f) => f.base10_digits().to_string(),
                syn::Lit::Str(s) => format!("\"{}\"", s.value()),
                _ => quote::quote!(#lit).to_string(),
            }
        }
        Expr::Path(path) => {
            if let Some(ident) = path.path.get_ident() {
                let name = ident.to_string();
                if name == "self" { "selfObj".to_string() } else { name }
            } else {
                ts_path_to_string(&path.path)
            }
        }
        Expr::MethodCall(mc) => {
            let recv = convert_expr(&mc.receiver);
            let method = mc.method.to_string();
            let args: Vec<String> = mc.args.iter().map(convert_expr).collect();
            format!("{}.{}({})", recv, method, args.join(", "))
        }
        Expr::Field(f) => {
            let base = convert_expr(&f.base);
            let member = match &f.member {
                syn::Member::Named(id) => id.to_string(),
                syn::Member::Unnamed(u) => u.index.to_string(),
            };
            format!("{}.{}", base, member)
        }
        Expr::Binary(binary) => {
            let left = convert_expr(&binary.left);
            let right = convert_expr(&binary.right);
            let op = match binary.op {
                syn::BinOp::Add(_) => "+",
                syn::BinOp::Sub(_) => "-",
                syn::BinOp::Mul(_) => "*",
                syn::BinOp::Div(_) => "/",
                syn::BinOp::Rem(_) => "%",
                syn::BinOp::And(_) => "&&",
                syn::BinOp::Or(_) => "||",
                syn::BinOp::Eq(_) => "===",
                syn::BinOp::Ne(_) => "!==",
                syn::BinOp::Lt(_) => "<",
                syn::BinOp::Le(_) => "<=",
                syn::BinOp::Gt(_) => ">",
                syn::BinOp::Ge(_) => ">=",
                _ => "/* unknown op */",
            };
            format!("{} {} {}", left, op, right)
        }
        Expr::Call(call) => {
            let func = convert_expr(&call.func);
            let args: Vec<String> = call.args.iter().map(|arg| convert_expr(arg)).collect();
            format!("{}({})", func, args.join(", "))
        }
        Expr::Macro(mac) => {
            // Handle macros like println!
            let macro_name = mac.mac.path.segments.last().unwrap().ident.to_string();
            match macro_name.as_str() {
                "println" => {
                    let tokens = mac.mac.tokens.to_string();
                    let cleaned = tokens.trim();
                    if cleaned.is_empty() {
                        "console.log()".to_string()
                    } else {
                        format!("console.log({})", cleaned)
                    }
                }
                "print" => {
                    let tokens = mac.mac.tokens.to_string();
                    let cleaned = tokens.trim();
                    format!("process.stdout.write({})", cleaned)
                }
                _ => format!("/* Unsupported macro: {}! */", macro_name)
            }
        }
        Expr::Return(ret) => {
            if let Some(expr) = &ret.expr {
                format!("return {}", convert_expr(expr))
            } else {
                "return".to_string()
            }
        }
        _ => "/* Unsupported expression */".to_string(),
    }
}

fn convert_struct(struct_item: &ItemStruct) -> String {
    let mut output = String::new();
    
    // Add comment showing original Rust struct
    output.push_str(&format!("// Converted from Rust: struct {}\n", struct_item.ident));
    // Capture generics like <T>
    let generics: Vec<String> = struct_item
        .generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect();
    let generic_suffix = if generics.is_empty() { String::new() } else { format!("<{}>", generics.join(", ")) };

    output.push_str(&format!("interface {}{} {{\n", struct_item.ident, generic_suffix));
    
    if let Fields::Named(fields) = &struct_item.fields {
        for field in &fields.named {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = convert_type(&field.ty);
            output.push_str(&format!("  {}: {};\n", field_name, field_type));
        }
    }
    
    output.push_str("}\n\n");
    output
}

fn convert_type(ty: &Type) -> String {
    match ty {
        Type::Reference(r) => {
            // &T or &mut T -> just T in TS
            convert_type(&r.elem)
        }
        Type::Path(type_path) => {
            let segment = &type_path.path.segments.last().unwrap();
            let ident = &segment.ident.to_string();
            
            match ident.as_str() {
                "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "usize" | "isize" => "number".to_string(),
                "String" | "str" => "string".to_string(),
                "bool" => "boolean".to_string(),
                "Self" => "any".to_string(),
                "Vec" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return format!("{}[]", convert_type(inner_ty));
                        }
                    }
                    "any[]".to_string()
                }
                "Option" => {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first() {
                            return format!("{} | undefined", convert_type(inner_ty));
                        }
                    }
                    "any | undefined".to_string()
                }
                _ => ident.clone(),
            }
        }
        _ => "any".to_string(),
    }
}

fn ts_path_to_string(path: &syn::Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for seg in &path.segments {
        parts.push(seg.ident.to_string());
    }
    parts.join(".")
}

fn convert_impl_inherent(impl_item: &syn::ItemImpl) -> String {
    // Determine the type name being implemented
    let mut out = String::new();
    let type_name = match &*impl_item.self_ty {
        Type::Path(tp) => tp.path.segments.last().map(|s| s.ident.to_string()).unwrap_or_else(|| "Self".to_string()),
        _ => "Self".to_string(),
    };

    // Collect generic type params like <T>
    let generics: Vec<String> = impl_item
        .generics
        .type_params()
        .map(|tp| tp.ident.to_string())
        .collect();
    let generic_suffix = if generics.is_empty() { String::new() } else { format!("<{}>", generics.join(", ")) };

    for it in &impl_item.items {
        if let ImplItem::Fn(meth) = it {
            // Function name: Type_method
            let fn_name = format!("{}_{}", type_name, meth.sig.ident);

            // Collect generics: impl generics + method generics
            let method_generics: Vec<String> = meth
                .sig
                .generics
                .type_params()
                .map(|tp| tp.ident.to_string())
                .collect();
            let mut all_generics: Vec<String> = generics.clone();
            for g in method_generics {
                if !all_generics.iter().any(|e| e == &g) {
                    all_generics.push(g);
                }
            }
            let fn_generic_suffix = if all_generics.is_empty() { String::new() } else { format!("<{}>", all_generics.join(", ")) };

            // Parameters: include explicit self if present
            let mut params: Vec<String> = Vec::new();
            // Receiver
            if let Some(receiver) = meth.sig.receiver() {
                let self_ty = format!("{}{}", type_name, generic_suffix);
                // &self or &mut self -> pass as first param
                let self_param = if receiver.mutability.is_some() {
                    format!("selfObj: {}", self_ty)
                } else {
                    format!("selfObj: {}", self_ty)
                };
                params.push(self_param);
            }

            // Other params
            for input in meth.sig.inputs.iter() {
                if let FnArg::Typed(pat_type) = input {
                    let param_name = match &*pat_type.pat {
                        syn::Pat::Ident(ident) => ident.ident.to_string(),
                        _ => quote::quote!(#pat_type.pat).to_string(),
                    };
                    let param_type = convert_type(&pat_type.ty);
                    params.push(format!("{}: {}", param_name, param_type));
                }
            }

            // Return type
            let ret_type = match &meth.sig.output {
                ReturnType::Type(_, ty) => format!(": {}", convert_type(ty)),
                ReturnType::Default => ": void".to_string(),
            };

            out.push_str(&format!("// Converted from Rust: impl {}{}::{}\n", type_name, generic_suffix, meth.sig.ident));
            out.push_str(&format!("function {}{}({}){} {{\n", fn_name, fn_generic_suffix, params.join(", "), ret_type));
            out.push_str(&convert_block(&meth.block));
            out.push_str("}\n\n");
        }
    }

    out
}

/// Convert a single `.rs` file to a `.ts` file written next to it.
/// Returns the path to the written `.ts` file.
pub fn convert_rust_file_to_ts_file<P: AsRef<Path>>(rs_path: P) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let rs_path = rs_path.as_ref();

    if !rs_path.is_file() {
        return Err(format!("Not a file: {}", rs_path.display()).into());
    }
    if rs_path.extension().and_then(|s| s.to_str()) != Some("rs") {
        return Err(format!("Not a .rs file: {}", rs_path.display()).into());
    }

    let source = fs::read_to_string(rs_path)?;
    let ast: File = syn::parse_file(&source)?;
    let ts = convert_rust_to_ts(&ast, &source);

    let mut ts_path = rs_path.to_path_buf();
    ts_path.set_extension("ts");

    fs::write(&ts_path, ts)?;
    Ok(ts_path)
}

fn is_ignored_dir(name: &str) -> bool {
    matches!(name, "target" | ".git" | "node_modules")
}

/// Convert all `.rs` files under `root` recursively (or a single file) to `.ts` files next to them.
/// Returns the list of `.ts` files written.
pub fn convert_rs_dir_to_ts_side_by_side<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let root = root.as_ref();
    let mut written = Vec::new();

    if root.is_file() {
        if root.extension().and_then(|s| s.to_str()) == Some("rs") {
            written.push(convert_rust_file_to_ts_file(root)?);
        }
        return Ok(written);
    }

    if !root.is_dir() {
        return Err(format!("Path not found or not accessible: {}", root.display()).into());
    }

    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();

            if path.is_dir() {
                if is_ignored_dir(&name) {
                    continue;
                }
                stack.push(path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let ts_path = convert_rust_file_to_ts_file(&path)?;
                written.push(ts_path);
            }
        }
    }

    Ok(written)
}
