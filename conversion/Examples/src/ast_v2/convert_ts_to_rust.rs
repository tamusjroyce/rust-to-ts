use std::path::Path;

use crate::ast_v2::rust_ast::module_to_rust;
use crate::ast_v2::ts_ast::from_ts_module;

pub fn convert_ts_file_to_rust(path: &Path) -> Result<String, String> {
    let module = from_ts_module(path)?;
    Ok(module_to_rust(&module))
}
