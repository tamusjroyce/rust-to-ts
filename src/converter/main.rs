#![deny(unused_mut)]

use std::env;
use std::fs;
use std::path::PathBuf;

#[path = "../ast/mod.rs"]
mod ast;
mod converter;

#[path = "../ast_v2/mod.rs"]
mod ast_v2;

fn main() {
    // BPMN smoke-test mode (runs regardless of --force).
    // Usage: rust-to-ts --bpmn-roundtrip <file.bpmn|file.xml>
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.len() >= 2 && raw_args[0] == "--bpmn-roundtrip" {
        let path = PathBuf::from(&raw_args[1]);
        let xml = fs::read_to_string(&path)
            .unwrap_or_else(|e| {
                eprintln!("Failed to read BPMN file {}: {}", path.display(), e);
                std::process::exit(1);
            });

        let rust = match ast_v2::bpmn::convert_bpmn_xml_to_rust_code(&xml) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("BPMN -> Rust failed: {}", e);
                std::process::exit(1);
            }
        };

        let xml2 = match ast_v2::bpmn::convert_rust_code_to_bpmn_xml(&rust) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Rust -> BPMN failed: {}", e);
                std::process::exit(1);
            }
        };

        if let Err(e) = ast_v2::bpmn::validate_bpmn_xml(&xml2) {
            eprintln!("Roundtrip BPMN invalid: {}", e);
            std::process::exit(1);
        }

        println!("--- BPMN -> Rust ---\n{}", rust);
        println!("--- Rust -> BPMN ---\n{}", xml2);
        return;
    }

    // Require an explicit opt-in before running the classic converter.
    // This binary is deprecated in favor of ast_v2; by default it exits
    // without doing any work unless --force is supplied.
    let force = raw_args.iter().any(|a| a == "--force");

    if !force {
        eprintln!(
            "[rust-to-ts] Classic converter is deprecated. \
pass --force to run it anyway, or use ast-v2 instead."
        );
        std::process::exit(2);
    }

    // Strip the --force flag from the remaining arguments.
    let args: Vec<String> = raw_args
        .into_iter()
        .filter(|a| a != "--force")
        .collect();

    // If no args (after removing --force), default to ./Examples
    let roots: Vec<PathBuf> = if args.is_empty() {
        vec![PathBuf::from("Examples")]
    } else {
        args.iter().map(PathBuf::from).collect()
    };

    let mut total = 0usize;
    let mut had_error = false;

    for root in roots {
        match converter::convert_rs_dir_to_ts_side_by_side(&root) {
            Ok(paths) => {
                for p in &paths {
                    println!("Wrote {}", p.display());
                }
                total += paths.len();
            }
            Err(e) => {
                eprintln!("Error converting {}: {}", root.display(), e);
                had_error = true;
            }
        }
    }

    println!("Converted {} file(s).", total);
    if had_error {
        std::process::exit(1);
    }
}
