#![deny(unused_mut)]

use std::env;
use std::path::Path;

// Pull in the refactored ast_v2 module from src/ast_v2 so this
// standalone dry-run harness can share the same AST and converters.
#[path = "../ast_v2/mod.rs"]
mod ast_v2;

use ast_v2::{from_rust_module, from_ts_module, module_to_rust, module_to_ts};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: ast-v2-dry-run <file.rs|file.ts> [...]");
        eprintln!("  Parses each file via ast_v2 and prints a simple summary.");
        std::process::exit(2);
    }

    let mut had_error = false;

    for arg in args {
        let path = Path::new(&arg);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let res = match ext.as_str() {
            "rs" => handle_rust_file(path),
            "ts" => handle_ts_file(path),
            "bpmn" | "xml" => handle_bpmn_file(path),
            _ => {
                eprintln!("[ast_v2_dry_run] Skipping {} (unsupported extension)", path.display());
                Ok(())
            }
        };

        if let Err(e) = res {
            eprintln!("[ast_v2_dry_run] {}: {}", path.display(), e);
            had_error = true;
        }
    }

    if had_error {
        std::process::exit(1);
    }
}

fn handle_rust_file(path: &Path) -> Result<(), String> {
    let module = from_rust_module(path)?;
    println!(
        "[ast_v2_dry_run] Rust module '{}' -> types: {}, functions: {}",
        module.name,
        module.types.len(),
        module.functions.len()
    );

    // Show the generated TS for quick inspection.
    let ts_code = module_to_ts(&module);
    println!("--- Generated TS for {} ---\n{}", path.display(), ts_code);
    Ok(())
}

fn handle_ts_file(path: &Path) -> Result<(), String> {
    let module = from_ts_module(path)?;
    println!(
        "[ast_v2_dry_run] TS module '{}' -> types: {}, functions: {}",
        module.name,
        module.types.len(),
        module.functions.len()
    );

    // Show the generated Rust for quick inspection.
    let rs_code = module_to_rust(&module);
    println!("--- Generated Rust for {} ---\n{}", path.display(), rs_code);
    Ok(())
}

fn handle_bpmn_file(path: &Path) -> Result<(), String> {
    let xml = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read BPMN file {}: {}", path.display(), e))?;

    let rust_code = ast_v2::bpmn::convert_bpmn_xml_to_rust_code(&xml)?;
    println!("[ast_v2_dry_run] BPMN {} -> Generated Rust:\n{}", path.display(), rust_code);

    let bpmn_xml = ast_v2::bpmn::convert_rust_code_to_bpmn_xml(&rust_code)?;
    // Validate the emitted XML parses.
    ast_v2::bpmn::validate_bpmn_xml(&bpmn_xml)?;
    println!("[ast_v2_dry_run] Rust -> BPMN (roundtrip) for {}:\n{}", path.display(), bpmn_xml);
    Ok(())
}
