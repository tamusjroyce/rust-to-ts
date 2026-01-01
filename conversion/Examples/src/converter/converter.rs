fn convert_rust_to_ts(ast: File, _source: String, is_main_file: bool) -> String {
    unimplemented!();
}

fn convert_function(func: ItemFn) -> String {
    unimplemented!();
}

fn convert_block(block: Block) -> String {
    unimplemented!();
}

fn convert_stmt(stmt: Stmt) -> String {
    unimplemented!();
}

fn convert_expr(expr: Expr) -> String {
    unimplemented!();
}

fn convert_struct(struct_item: ItemStruct) -> String {
    unimplemented!();
}

fn convert_type(ty: Type) -> String {
    unimplemented!();
}

fn convert_ir_type(ir: IrType) -> String {
    unimplemented!();
}

fn ts_path_to_string(path: Path) -> String {
    unimplemented!();
}

fn convert_impl_inherent(impl_item: ItemImpl) -> String {
    unimplemented!();
}

fn convert_macro_stmt(stmt_mac: StmtMacro) -> String {
    unimplemented!();
}

fn convert_macro_expr(mac: ExprMacro) -> String {
    unimplemented!();
}

fn parse_format_macro_args(tokens: TokenStream) -> any {
    unimplemented!();
}

fn build_ts_template_string(fmt: String, args: any) -> String {
    unimplemented!();
}

fn convert_for_loop(fl: ExprForLoop) -> String {
    unimplemented!();
}

fn convert_if_stmt(ifexpr: ExprIf) -> String {
    unimplemented!();
}

fn convert_rust_file_to_ts_file<P>(rs_path: P) -> Result {
    unimplemented!();
}

fn is_ignored_dir(name: String) -> bool {
    unimplemented!();
}

fn convert_rs_dir_to_ts_side_by_side<P>(root: P) -> Result {
    unimplemented!();
}
