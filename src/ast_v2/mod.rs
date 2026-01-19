pub mod ast;
pub mod bpmn;
pub mod rust;
pub mod typescript;
pub mod convert_rust_to_ts;
pub mod convert_ts_to_rust;
pub mod tagging;
#[allow(unused_imports)]
pub use ast::{Field, Function, FunctionKind, Module, Param, TypeDecl, TypeKind, TypeRef};
pub use convert_rust_to_ts::convert_rust_file_to_ts;
pub use convert_ts_to_rust::convert_ts_file_to_rust;
#[allow(unused_imports)]
pub use rust::{from_rust_module, module_to_rust};
#[allow(unused_imports)]
pub use typescript::{from_ts_module, module_to_ts};

#[allow(dead_code)]
pub fn compare_and_print(rust_mod: &Module, ts_mod: &Module) {
    println!("=== ast_v2 Rust module ===\n{:#?}", rust_mod);
    println!("=== ast_v2 TS module ===\n{:#?}", ts_mod);

    println!("=== ast_v2 HelloWorld mapping ===");

    for rtype in &rust_mod.types {
        if let Some(ttype) = ts_mod.types.iter().find(|t| t.name == rtype.name) {
            println!(
                "Type {}: Rust {:?} ↔ TS {:?}",
                rtype.name, rtype.kind, ttype.kind
            );
            for rf in &rtype.fields {
                match ttype.fields.iter().find(|tf| tf.name == rf.name) {
                    Some(tf) => {
                        let status = if rf.ty == tf.ty { "OK" } else { "MISMATCH" };
                        println!(
                            "  field {}: Rust {:?} ↔ TS {:?} => {}",
                            rf.name, rf.ty, tf.ty, status
                        );
                    }
                    None => println!(
                        "  field {}: present in Rust, missing in TS",
                        rf.name
                    ),
                }
            }
        } else {
            println!("Type {}: present in Rust, missing in TS", rtype.name);
        }
    }

    for rfn in &rust_mod.functions {
        if let Some(tfn) = ts_mod.functions.iter().find(|f| f.name == rfn.name) {
            println!("Function {}:", rfn.name);
            let param_status = if rfn.params.len() == tfn.params.len()
                && rfn
                    .params
                    .iter()
                    .zip(&tfn.params)
                    .all(|(rp, tp)| rp.name == tp.name && rp.ty == tp.ty)
            {
                "params OK"
            } else {
                "params MISMATCH"
            };

            let ret_status = if rfn.return_type == tfn.return_type {
                "return type OK"
            } else {
                "return type MISMATCH"
            };

            println!("  {} / {}", param_status, ret_status);
        } else {
            println!("Function {}: present in Rust, missing in TS", rfn.name);
        }
    }
}
