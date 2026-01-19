fn convert_rust_to_ts(ast: File, _source: String, is_main_file: bool) -> String {
    Default::default()
}

fn convert_function(func: ItemFn) -> String {
    Default::default()
}

fn convert_block(block: Block) -> String {
    Default::default()
}

fn convert_stmt(stmt: Stmt) -> String {
    Default::default()
}

fn convert_expr(expr: Expr) -> String {
    Default::default()
}

fn convert_struct(struct_item: ItemStruct) -> String {
    Default::default()
}

fn convert_type(ty: Type) -> String {
    let ir = rust_type_to_ir(ty);
    convert_ir_type(&ir)
}

fn convert_ir_type(ir: &IrType) -> String {
    match ir {
        IrType::Number => "number".to_string(),
        IrType::String => "string".to_string(),
        IrType::Bool => "boolean".to_string(),
        IrType::Any => "any".to_string(),
        IrType::Vec(inner) => format!("{}[]", convert_ir_type(inner)),
        IrType::Option(inner) => format!("{} | undefined", convert_ir_type(inner)),
        IrType::Custom(name) => name.clone(),
    }
}

fn ts_path_to_string(path: &syn::Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for seg in &path.segments {
        parts.push(seg.ident.to_string());
    }
    parts.join(".")
}

fn convert_impl_inherent(impl_item: ItemImpl) -> String {
    Default::default()
}

fn convert_macro_stmt(stmt_mac: StmtMacro) -> String {
    Default::default()
}

fn convert_macro_expr(mac: ExprMacro) -> String {
    Default::default()
}

fn parse_format_macro_args(tokens: TokenStream) -> String {
    Default::default()
}

fn build_ts_template_string(fmt: String, args: String) -> String {
    Default::default()
}

fn convert_for_loop(fl: ExprForLoop) -> String {
    Default::default()
}

fn convert_if_stmt(ifexpr: ExprIf) -> String {
    Default::default()
}

fn convert_rust_file_to_ts_file<P>(rs_path: P) -> Result {
    Default::default()
}

fn is_ignored_dir(name: String) -> bool {
    Default::default()
}

fn convert_rs_dir_to_ts_side_by_side<P>(root: P) -> Result {
    Default::default()
}
