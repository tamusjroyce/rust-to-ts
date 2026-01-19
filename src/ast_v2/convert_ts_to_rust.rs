use std::path::Path;

use crate::ast_v2::{from_ts_module, module_to_rust};

pub fn convert_ts_file_to_rust(path: &Path) -> Result<String, String> {
    let module = from_ts_module(path)?;
    Ok(module_to_rust(&module))
}
