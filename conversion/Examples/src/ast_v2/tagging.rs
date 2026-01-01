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
