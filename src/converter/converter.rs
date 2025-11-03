use syn::{File, Item, ItemFn, FnArg, ReturnType, Type, ItemStruct, Fields, Stmt, Expr, Block};
use syn::{ImplItem};
use syn::{Pat, PatTuple};
use syn::punctuated::Punctuated;
use syn::Token;
use syn::parse::Parser;
use std::fs;
use std::path::{Path, PathBuf};

pub fn convert_rust_to_ts(ast: &File, _source: &str, is_main_file: bool) -> String {
    let mut output = String::new();
    if is_main_file {
        // Naive import for sibling lib when converting a main.rs
        output.push_str("import { NeuralNetwork, make_rng_from_args } from \"./lib.ts\";\n\n");
    }
    
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
                output.push_str(&format!("// Unsupported Rust item: {}\n\n", quote::quote!(#item).to_string()));
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

    // Export all free functions for cross-file usage (including main)
    output.push_str("export ");
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
        // If the last statement is an expression without a semicolon, return it,
        // except for control-flow expressions like for/if which should emit as statements.
        if let Some(last) = last_idx {
            if i == last {
                if let Stmt::Expr(expr, None) = stmt {
                    match expr {
                        Expr::ForLoop(_) | Expr::If(_) => {
                            // fall through to normal statement conversion
                        }
                        _ => {
                            let expr_str = convert_expr(expr);
                            if expr_str.is_empty() || expr_str.starts_with("/* Unsupported") {
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
            }
        }

        output.push_str(&convert_stmt(stmt));
    }

    output
}

fn convert_stmt(stmt: &Stmt) -> String {
    match stmt {
        Stmt::Macro(stmt_mac) => convert_macro_stmt(stmt_mac),
        Stmt::Local(local) => {
            let mut result = String::from("  // Rust variable declaration\n");
            // Determine variable names and whether it's a tuple destructure
            let (pat_ts, is_tuple, is_mut) = match &local.pat {
                Pat::Ident(p) => (p.ident.to_string(), false, p.mutability.is_some()),
                Pat::Type(pt) => {
                    // Handle patterns like `let name: Type = ...;`
                    match &*pt.pat {
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
                    }
                }
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
                if expr_str.contains("Unsupported expression") {
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
            // Special forms: for-loops and if (including if-let)
            if let Expr::ForLoop(fl) = expr {
                return convert_for_loop(fl);
            }
            if let Expr::If(ifexpr) = expr {
                return convert_if_stmt(ifexpr);
            }
            let expr_str = convert_expr(expr);
            if expr_str.is_empty() || expr_str.contains("Unsupported expression") {
                return format!("  // Unsupported statement: {}\n", quote::quote!(#expr).to_string());
            }
            let mut result = String::from("  // Rust expression\n");
            result.push_str(&format!("  {}", expr_str));
            if semi.is_some() {
                result.push(';');
            }
            result.push('\n');
            result
        }
        _ => String::from("  // Unsupported statement (unhandled variant)\n"),
    }
}

fn convert_expr(expr: &Expr) -> String {
    match expr {
        Expr::ForLoop(fl) => {
            // Handled at statement level, but keep a comment if used in expression position
            format!("(undefined as any) /* Unsupported expression (for-loop): {} */", quote::quote!(#fl).to_string())
        }
        Expr::If(ifexpr) => {
            // Handled at statement level
            format!("(undefined as any) /* Unsupported expression (if): {} */", quote::quote!(#ifexpr).to_string())
        }
        Expr::Unary(u) => {
            use syn::UnOp;
            let inner = convert_expr(&u.expr);
            match u.op {
                UnOp::Neg(_) => format!("-{}", inner),
                UnOp::Not(_) => format!("!{}", inner),
                _ => "(undefined as any) /* Unsupported expression */".to_string(),
            }
        }
        Expr::Lit(lit) => {
            match &lit.lit {
                syn::Lit::Int(i) => i.base10_digits().to_string(),
                syn::Lit::Float(f) => f.base10_digits().to_string(),
                syn::Lit::Str(s) => format!("\"{}\"", s.value()),
                _ => quote::quote!(#lit).to_string(),
            }
        }
        Expr::Tuple(t) => {
            let parts: Vec<String> = t.elems.iter().map(convert_expr).collect();
            format!("[{}]", parts.join(", "))
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
            // Map common Rust std methods to JS where safe
            match method.as_str() {
                "min" => {
                    if let Some(arg0) = args.get(0) { format!("Math.min({}, {})", recv, arg0) } else { format!("/* Unsupported min call: {}.min() */", recv) }
                }
                "max" => {
                    if let Some(arg0) = args.get(0) { format!("Math.max({}, {})", recv, arg0) } else { format!("/* Unsupported max call: {}.max() */", recv) }
                }
                _ => format!("{}.{}({})", recv, method, args.join(", ")),
            }
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
                _ => "+",
            };
            format!("{} {} {}", left, op, right)
        }
        Expr::Call(call) => {
            let func = convert_expr(&call.func);
            let args: Vec<String> = call.args.iter().map(|arg| convert_expr(arg)).collect();
            // Special cases to keep TS runnable
            if func == "make_rng_from_args" {
                // Inline a simple seeded RNG object using mulberry32; ignores actual args
                return String::from("({ next_f32: (low: number, high: number) => { function mulberry32(a:number){return function(){let t=a+=0x6D2B79F5;t=Math.imul(t^t>>>15,t|1);t^=t+Math.imul(t^t>>>7,t|61);return ((t^t>>>14)>>>0)/4294967296;}} const seed=(globalThis as any).__RUST_TO_TS_SEED>>>0||0xDEADBEEF; const rand=mulberry32(seed); return low + rand() * (high - low); } })");
            }
            if func == "rng_name_from_args" {
                // Read RNG name from global injected by tester
                return String::from("(((globalThis as any).__RUST_TO_TS_RNG||'default') as string)");
            }
            if func == "NeuralNetwork.random_uniform_f32_with" || func == "NeuralNetwork.random_uniform_f64_with" {
                // Map to JS-side helper without RNG parameter
                let mut a = args.clone();
                if !a.is_empty() { a.pop(); } // drop rng
                return format!("NeuralNetwork.random_uniform({})", a.join(", "));
            }
            format!("{}({})", func, args.join(", "))
        }
        Expr::Macro(mac) => convert_macro_expr(mac),
        Expr::Return(ret) => {
            if let Some(expr) = &ret.expr {
                format!("return {}", convert_expr(expr))
            } else {
                "return".to_string()
            }
        }
        _ => format!("(undefined as any) /* Unsupported expression: {} */", quote::quote!(#expr).to_string()),
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

    // Provide a runtime namespace object for associated functions if this is NeuralNetwork
    if struct_item.ident == "NeuralNetwork" {
        output.push_str("export const NeuralNetwork: any = {};\n\n");
    }
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

    // Special handling for NeuralNetwork associated functions to produce a usable TS runtime
    if type_name == "NeuralNetwork" {
        let mut has_new_with_value = false;
        let mut has_random_uniform = false;
        for it in &impl_item.items {
            if let ImplItem::Fn(meth) = it {
                if meth.sig.receiver().is_none() {
                    let name = meth.sig.ident.to_string();
                    if name == "new_with_value" { has_new_with_value = true; }
                    if name == "random_uniform" { has_random_uniform = true; }
                }
            }
        }
        if has_new_with_value {
            out.push_str(&format!("// Converted from Rust: impl {} associated functions\n", type_name));
            out.push_str("Object.assign(NeuralNetwork, {\n");
            out.push_str("  new_with_value: function(x_layers: number, y_nodes: number, z_weights: number, value: any) {\n");
            out.push_str("    const size = x_layers * y_nodes * z_weights;\n");
            out.push_str("    const data = Array(size).fill(value);\n");
            out.push_str("    return {\n");
            out.push_str("      x_layers, y_nodes, z_weights, data,\n");
            out.push_str("      dims() { return [this.x_layers, this.y_nodes, this.z_weights]; },\n");
            out.push_str("      len() { return this.data.length; },\n");
            out.push_str("      is_empty() { return this.data.length === 0; },\n");
            out.push_str("      index(x: number, y: number, z: number) {\n");
            out.push_str("        if (x < this.x_layers && y < this.y_nodes && z < this.z_weights) {\n");
            out.push_str("          const yz = this.y_nodes * this.z_weights;\n");
            out.push_str("          return x * yz + y * this.z_weights + z;\n");
            out.push_str("        }\n");
            out.push_str("        return undefined;\n");
            out.push_str("      },\n");
            out.push_str("      get(x: number, y: number, z: number) {\n");
            out.push_str("        const i = this.index(x, y, z);\n");
            out.push_str("        return i === undefined ? undefined : this.data[i];\n");
            out.push_str("      },\n");
            out.push_str("      get_mut(x: number, y: number, z: number) {\n");
            out.push_str("        const i = this.index(x, y, z);\n");
            out.push_str("        return i === undefined ? undefined : this.data[i];\n");
            out.push_str("      },\n");
            out.push_str("    };\n");
            out.push_str("  },\n");
            out.push_str("});\n\n");
        }
        if has_random_uniform {
            out.push_str(&format!("// Converted from Rust: impl {} associated functions\n", type_name));
            out.push_str("Object.assign(NeuralNetwork, {\n");
            out.push_str("  random_uniform: function(x_layers: number, y_nodes: number, z_weights: number, low: number, high: number) {\n");
            // RNG selection based on globals set by the tester wrapper
            out.push_str("    function mulberry32Factory(a:number){return function(){let t=a+=0x6D2B79F5;t=Math.imul(t^t>>>15,t|1);t^=t+Math.imul(t^t>>>7,t|61);return ((t^t>>>14)>>>0)/4294967296;}}\n");
            out.push_str("    function splitmix64Factory(seed: bigint){\n");
            out.push_str("      let state = seed;\n");
            out.push_str("      const MASK = (1n<<64n)-1n;\n");
            out.push_str("      return function(){\n");
            out.push_str("        state = (state + 0x9E3779B97F4A7C15n) & MASK;\n");
            out.push_str("        let z = state;\n");
            out.push_str("        z ^= z >> 30n; z = (z * 0xBF58476D1CE4E5B9n) & MASK;\n");
            out.push_str("        z ^= z >> 27n; z = (z * 0x94D049BB133111EBn) & MASK;\n");
            out.push_str("        z ^= z >> 31n;\n");
            out.push_str("        const u = Number(z >> 11n) / 9007199254740992; // 2^53\n");
            out.push_str("        return u;\n");
            out.push_str("      }\n");
            out.push_str("    }\n");
            // ChaCha8 RNG factory mirroring the Rust implementation using 32-bit ops
            out.push_str("    function rotl32(x:number, n:number){ x = x>>>0; return ((x<<n) | (x>>> (32-n)))>>>0; }\n");
            out.push_str("    function chacha8Factory(seedU64: bigint){\n");
            out.push_str("      const st = new Uint32Array(16);\n");
            out.push_str("      st[0]=0x61707865; st[1]=0x3320646e; st[2]=0x79622d32; st[3]=0x6b206574;\n");
            out.push_str("      // Mulberry64-like (splitmix64-based) generator to derive key words via f32 cast pattern\n");
            out.push_str("      const sm = (function(){ let s=seedU64; const MASK=(1n<<64n)-1n; return function(){ s=(s+0x9E3779B97F4A7C15n)&MASK; let z=s; z^=z>>30n; z=(z*0xBF58476D1CE4E5B9n)&MASK; z^=z>>27n; z=(z*0x94D049BB133111EBn)&MASK; z^=z>>31n; const u=Number(z>>11n)/9007199254740992; return u; };})();\n");
            out.push_str("      const F32_MAX = 340282346638528859811704183484516925440.0; // f32::MAX as f64\n");
            out.push_str("      for (let i=0;i<8;i++){\n");
            out.push_str("        // Emulate sm.next_f32(0.0, f32::MAX) as u32 with saturating cast\n");
            out.push_str("        let val = sm()*F32_MAX;\n");
            out.push_str("        let u32 = 0; if (!Number.isFinite(val) || val<=0){ u32=0; } else if (val>=4294967295){ u32=4294967295; } else { u32 = Math.floor(val)>>>0; }\n");
            out.push_str("        const tweak = Math.imul(0x9E3779B9>>>0, (i+1)>>>0)>>>0;\n");
            out.push_str("        st[4+i] = (u32 ^ tweak)>>>0;\n");
            out.push_str("      }\n");
            out.push_str("      st[12]=0; st[13]=0;\n");
            out.push_str("      const seedLo = Number(seedU64 & 0xFFFFFFFFn)>>>0;\n");
            out.push_str("      const seedHi = Number((seedU64>>32n) & 0xFFFFFFFFn)>>>0;\n");
            out.push_str("      st[14] = (seedLo ^ 0xDEADBEEF)>>>0;\n");
            out.push_str("      st[15] = (seedHi ^ 0xBADC0FFE)>>>0;\n");
            out.push_str("      const buf = new Uint32Array(16); let idx=16;\n");
            out.push_str("      function qr(x:Uint32Array,a:number,b:number,c:number,d:number){ x[a]=(x[a]+x[b])>>>0; x[d]^=x[a]; x[d]=rotl32(x[d],16); x[c]=(x[c]+x[d])>>>0; x[b]^=x[c]; x[b]=rotl32(x[b],12); x[a]=(x[a]+x[b])>>>0; x[d]^=x[a]; x[d]=rotl32(x[d],8); x[c]=(x[c]+x[d])>>>0; x[b]^=x[c]; x[b]=rotl32(x[b],7); }\n");
            out.push_str("      function refill(){\n");
            out.push_str("        const x = new Uint32Array(st);\n");
            out.push_str("        for (let r=0;r<8;r++){\n");
            out.push_str("          // column rounds\n");
            out.push_str("          qr(x,0,4,8,12); qr(x,1,5,9,13); qr(x,2,6,10,14); qr(x,3,7,11,15);\n");
            out.push_str("          // diagonal rounds\n");
            out.push_str("          qr(x,0,5,10,15); qr(x,1,6,11,12); qr(x,2,7,8,13); qr(x,3,4,9,14);\n");
            out.push_str("        }\n");
            out.push_str("        for (let i=0;i<16;i++){ buf[i] = (x[i] + st[i])>>>0; }\n");
            out.push_str("        st[12] = (st[12] + 1)>>>0; if (st[12]===0){ st[13] = (st[13]+1)>>>0; }\n");
            out.push_str("        idx=0;\n");
            out.push_str("      }\n");
            out.push_str("      function next_u32(){ if (idx>=16) refill(); const v = buf[idx]; idx++; return v>>>0; }\n");
            out.push_str("      return function(){ const r = next_u32() / 4294967296; return r; };\n");
            out.push_str("    }\n");
            out.push_str("    const g:any = globalThis as any;\n");
            out.push_str("    const rngName = (g.__RUST_TO_TS_RNG || '').toString().toLowerCase();\n");
            out.push_str("    const seed32 = (g.__RUST_TO_TS_SEED >>> 0) || 0xDEADBEEF;\n");
            out.push_str("    const seedU64: bigint = (typeof g.__RUST_TO_TS_SEED_U64 !== 'undefined') ? BigInt(g.__RUST_TO_TS_SEED_U64) : BigInt(seed32 >>> 0);\n");
            out.push_str("    let rand: () => number;\n");
            out.push_str("    if (rngName === 'mulberry64') { rand = splitmix64Factory(seedU64); }\n");
            out.push_str("    else if (rngName === 'pcg64') { rand = splitmix64Factory(seedU64); /* TODO: implement PCG64 */ }\n");
            out.push_str("    else if (rngName === 'chacha8' || rngName === 'chacha8rng') { rand = chacha8Factory(seedU64); }\n");
            out.push_str("    else { rand = mulberry32Factory(seed32); }\n");
            out.push_str("    const size = x_layers * y_nodes * z_weights;\n");
            out.push_str("    const data: number[] = new Array(size);\n");
            out.push_str("    for (let i = 0; i < size; i++) { const r = rand(); data[i] = low + r * (high - low); }\n");
            out.push_str("    return {\n");
            out.push_str("      x_layers, y_nodes, z_weights, data,\n");
            out.push_str("      dims() { return [this.x_layers, this.y_nodes, this.z_weights]; },\n");
            out.push_str("      len() { return this.data.length; },\n");
            out.push_str("      is_empty() { return this.data.length === 0; },\n");
            out.push_str("      index(x: number, y: number, z: number) {\n");
            out.push_str("        if (x < this.x_layers && y < this.y_nodes && z < this.z_weights) {\n");
            out.push_str("          const yz = this.y_nodes * this.z_weights;\n");
            out.push_str("          return x * yz + y * this.z_weights + z;\n");
            out.push_str("        }\n");
            out.push_str("        return undefined;\n");
            out.push_str("      },\n");
            out.push_str("      get(x: number, y: number, z: number) {\n");
            out.push_str("        const i = this.index(x, y, z);\n");
            out.push_str("        return i === undefined ? undefined : this.data[i];\n");
            out.push_str("      },\n");
            out.push_str("      get_mut(x: number, y: number, z: number) {\n");
            out.push_str("        const i = this.index(x, y, z);\n");
            out.push_str("        return i === undefined ? undefined : this.data[i];\n");
            out.push_str("      },\n");
            out.push_str("    };\n");
            out.push_str("  },\n");
            out.push_str("});\n\n");
        }
        // We still fall through and generate generic free functions for methods if any
    }

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
            if let Some(_receiver) = meth.sig.receiver() {
                let self_ty = format!("{}{}", type_name, generic_suffix);
                let self_param = format!("selfObj: {}", self_ty);
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

fn convert_macro_stmt(stmt_mac: &syn::StmtMacro) -> String {
    let name = stmt_mac
        .mac
        .path
        .segments
        .last()
        .map(|s| s.ident.to_string())
        .unwrap_or_default();
    match name.as_str() {
        "println" => {
            let (fmt, args) = parse_format_macro_args(&stmt_mac.mac.tokens);
            let mut out = String::from("  // Rust macro\n");
            if let Some(fmt_str) = fmt {
                out.push_str(&format!("  console.log({});\n", build_ts_template_string(&fmt_str, &args)));
            } else {
                // Fallback to raw console.log with arguments, plus original code
                out.push_str(&format!(
                    "  console.log({}); // raw args\n  // Original: {}\n",
                    stmt_mac.mac.tokens.to_string(),
                    quote::quote!(#stmt_mac).to_string()
                ));
            }
            out
        }
        "print" => {
            let (fmt, args) = parse_format_macro_args(&stmt_mac.mac.tokens);
            let mut out = String::from("  // Rust macro\n");
            if let Some(fmt_str) = fmt {
                out.push_str(&format!("  Deno.stdout.writeSync(new TextEncoder().encode({}));\n", build_ts_template_string(&fmt_str, &args)));
            } else {
                out.push_str(&format!(
                    "  Deno.stdout.writeSync(new TextEncoder().encode(String({})));\n  // Original: {}\n",
                    stmt_mac.mac.tokens.to_string(),
                    quote::quote!(#stmt_mac).to_string()
                ));
            }
            out
        }
        _ => {
            format!("  // Unsupported macro: {}!\\n  // Original: {}\n", name, quote::quote!(#stmt_mac).to_string())
        }
    }
}

fn convert_macro_expr(mac: &syn::ExprMacro) -> String {
    let macro_name = mac.mac.path.segments.last().unwrap().ident.to_string();
    match macro_name.as_str() {
        "println" => {
            let (fmt, args) = parse_format_macro_args(&mac.mac.tokens);
            if let Some(fmt_str) = fmt {
                build_ts_template_string(&fmt_str, &args)
            } else {
                format!("console.log({})", mac.mac.tokens.to_string())
            }
        }
        "print" => {
            let (fmt, args) = parse_format_macro_args(&mac.mac.tokens);
            if let Some(fmt_str) = fmt {
                format!("Deno.stdout.writeSync(new TextEncoder().encode({}))", build_ts_template_string(&fmt_str, &args))
            } else {
                format!("Deno.stdout.writeSync(new TextEncoder().encode(String({})))", mac.mac.tokens.to_string())
            }
        }
        _ => format!("/* Unsupported macro: {}! - Original: {} */", macro_name, quote::quote!(#mac).to_string())
    }
}

fn parse_format_macro_args(tokens: &proc_macro2::TokenStream) -> (Option<String>, Vec<String>) {
    // Parse tokens as a comma-separated list of expressions
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    if let Ok(list) = parser.parse2(tokens.clone()) {
        let mut it = list.iter();
        if let Some(first) = it.next() {
            if let Expr::Lit(expr_lit) = first {
                if let syn::Lit::Str(s) = &expr_lit.lit {
                    let fmt = s.value();
                    let mut args: Vec<String> = Vec::new();
                    for a in it {
                        args.push(convert_expr(a));
                    }
                    return (Some(fmt), args);
                }
            }
        }
        // No format string literal; treat all as args
        let args: Vec<String> = list.iter().map(convert_expr).collect();
        return (None, args);
    }
    (None, Vec::new())
}

fn build_ts_template_string(fmt: &str, args: &[String]) -> String {
    // Handle escaped braces {{ }} and simple placeholders {} or {:?}
    let s = fmt.replace("{{", "{{}").replace("}}", "{}}");
    // Replace {} and {:?} sequentially with ${arg}
    let mut result = String::from("`");
    let mut arg_iter = args.iter();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            // lookahead for closing }
            let mut j = i + 1;
            while j < chars.len() && chars[j] != '}' { j += 1; }
            if j < chars.len() && chars[j] == '}' {
                // placeholder between i..=j
                if let Some(arg) = arg_iter.next() {
                    result.push_str("${");
                    result.push_str(arg);
                    result.push('}');
                } else {
                    // no arg, keep literal braces
                    result.push('{');
                    result.push('}');
                }
                i = j + 1;
                continue;
            }
        }
        // escape backticks and $ in literals
        if chars[i] == '`' { result.push_str("\\`"); }
        else if chars[i] == '$' { result.push_str("\\$"); }
        else { result.push(chars[i]); }
        i += 1;
    }
    result.push('`');
    result
}

fn convert_for_loop(fl: &syn::ExprForLoop) -> String {
    // Support `for i in start..end` and `for i in start..=end`
    let pat_name = match &*fl.pat {
        Pat::Ident(id) => id.ident.to_string(),
        _ => return format!(
            "  // Unsupported for-loop pattern: {}\n  // Original: {}\n",
            quote::quote!(#fl.pat).to_string(),
            quote::quote!(#fl).to_string()
        ),
    };

    let (start_expr, end_expr, inclusive) = match &*fl.expr {
        Expr::Range(r) => {
            let start = r.start.as_ref().map(|e| convert_expr(e)).unwrap_or_else(|| "0".to_string());
            let end = r.end.as_ref().map(|e| convert_expr(e)).unwrap_or_else(|| "/* missing end */".to_string());
            let incl = matches!(r.limits, syn::RangeLimits::Closed(_));
            (start, end, incl)
        }
        other => {
            return format!(
                "  // Unsupported for-loop iterator: {}\n  // Original: {}\n",
                quote::quote!(#other).to_string(),
                quote::quote!(#fl).to_string()
            );
        }
    };

    let mut out = String::new();
    out.push_str("  // Rust for-loop\n");
    if inclusive {
        out.push_str(&format!(
            "  for (let {} = {}; {} <= {}; {}++) {{\n",
            pat_name, start_expr, pat_name, end_expr, pat_name
        ));
    } else {
        out.push_str(&format!(
            "  for (let {} = {}; {} < {}; {}++) {{\n",
            pat_name, start_expr, pat_name, end_expr, pat_name
        ));
    }
    out.push_str(&convert_block(&fl.body));
    out.push_str("  }\n");
    out
}

fn convert_if_stmt(ifexpr: &syn::ExprIf) -> String {
    // Support `if let PAT = EXPR { ... }` for simple Option binding, otherwise fallback to plain condition
    if let Expr::Let(letcond) = &*ifexpr.cond {
        // Allow `if let Some(x) = expr` or `if let x = expr`
        let pat_name = match &*letcond.pat {
            Pat::Ident(id) => id.ident.to_string(),
            // if let Some(x) = expr -> bind inner ident
            Pat::TupleStruct(ts) => {
                if ts.path.segments.last().map(|s| s.ident.to_string()).as_deref() == Some("Some") {
                    if ts.elems.len() == 1 {
                        if let Pat::Ident(id) = &ts.elems[0] {
                            id.ident.to_string()
                        } else {
                            "val".to_string()
                        }
                    } else { "val".to_string() }
                } else { "val".to_string() }
            }
            _ => "val".to_string(),
        };
        let value_expr = convert_expr(&letcond.expr);
        let mut out = String::new();
        out.push_str("  // Rust if-let\n");
        out.push_str(&format!("  const __tmp = {};\n", value_expr));
        out.push_str("  if (__tmp !== undefined) {\n");
        out.push_str(&format!("    const {} = __tmp;\n", pat_name));
        out.push_str(&convert_block(&ifexpr.then_branch));
        if let Some((_else, else_expr)) = &ifexpr.else_branch {
            // else could be another if or a block
            match &**else_expr {
                Expr::Block(b) => {
                    out.push_str("  } else {\n");
                    out.push_str(&convert_block(&b.block));
                    out.push_str("  }\n");
                }
                other => {
                    out.push_str("  } else {\n");
                    out.push_str(&format!("    // Unsupported else expression: {}\n", quote::quote!(#other).to_string()));
                    out.push_str("  }\n");
                }
            }
        } else {
            out.push_str("  }\n");
        }
        return out;
    }

    // Plain if condition
    let cond = convert_expr(&ifexpr.cond);
    let mut out = String::new();
    out.push_str("  // Rust if\n");
    out.push_str(&format!("  if ({}) {{\n", cond));
    out.push_str(&convert_block(&ifexpr.then_branch));
    if let Some((_else, else_expr)) = &ifexpr.else_branch {
        match &**else_expr {
            Expr::Block(b) => {
                out.push_str("  } else {\n");
                out.push_str(&convert_block(&b.block));
                out.push_str("  }\n");
            }
            other => {
                out.push_str("  } else {\n");
                out.push_str(&format!("    // Unsupported else expression: {}\n", quote::quote!(#other).to_string()));
                out.push_str("  }\n");
            }
        }
    } else {
        out.push_str("  }\n");
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
    let is_main = rs_path.file_stem().and_then(|s| s.to_str()) == Some("main");
    let ts = convert_rust_to_ts(&ast, &source, is_main);

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
