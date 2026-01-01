use std::path::Path;

use crate::ast_v2::ast::{FunctionKind, Module};

pub fn tag_special_functions_for_path(module: &mut Module, path: &Path) {
    // HelloWorld: standalone hello_world.rs example
    if module.name == "hello_world" {
        tag_hello_world_functions(module);
        return;
    }

    // NeuralNetwork: Examples/NeuralNetwork/src/main.rs
    if module.name == "main" {
        let mut has_examples = false;
        let mut has_nn = false;
        let mut cur = path.parent();
        while let Some(p) = cur {
            if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                if name == "Examples" {
                    has_examples = true;
                }
                if name == "NeuralNetwork" {
                    has_nn = true;
                }
            }
            cur = p.parent();
        }

        if has_examples && has_nn {
            tag_neural_net_main(module);
        }
    }

    // ast_v2 self: src/ast_v2/convert_rust_to_ts.rs
    // Recognize the pipeline entry so TS emission can be explicit.
    if module.name == "convert_rust_to_ts" {
        tag_ast_v2_convert_rust_file_to_ts(module);
    }

    // ast_v2 self: src/ast_v2/convert_ts_to_rust.rs
    if module.name == "convert_ts_to_rust" {
        tag_ast_v2_convert_ts_file_to_rust(module);
    }
}

fn tag_hello_world_functions(module: &mut Module) {
    for func in &mut module.functions {
        match func.name.as_str() {
            "main" => func.kind = FunctionKind::HelloWorldMain,
            "add" if func.params.len() == 2 => func.kind = FunctionKind::HelloWorldAdd,
            _ => {}
        }
    }
}

fn tag_neural_net_main(module: &mut Module) {
    for func in &mut module.functions {
        if func.name == "main" && func.params.is_empty() {
            func.kind = FunctionKind::NeuralNetMain;
        }
    }
}

fn tag_ast_v2_convert_rust_file_to_ts(module: &mut Module) {
    for func in &mut module.functions {
        if func.name == "convert_rust_file_to_ts" {
            func.kind = FunctionKind::AstV2ConvertRustFileToTs;
        }
    }
}

fn tag_ast_v2_convert_ts_file_to_rust(module: &mut Module) {
    for func in &mut module.functions {
        if func.name == "convert_ts_file_to_rust" {
            func.kind = FunctionKind::AstV2ConvertTsFileToRust;
        }
    }
}
