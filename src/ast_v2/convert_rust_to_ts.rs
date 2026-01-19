use std::path::Path;

use crate::ast_v2::ast::Module;
use crate::ast_v2::{from_rust_module, module_to_ts};

use crate::ast_v2::tagging::tag_special_functions_for_path;

pub fn convert_rust_file_to_ts(path: &Path) -> Result<String, String> {
    let mut module: Module = from_rust_module(path)?;
    tag_special_functions_for_path(&mut module, path);
    Ok(module_to_ts(&module))
}
