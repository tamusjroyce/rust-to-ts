use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[path = "../ast_v2/mod.rs"]
mod ast_v2;

use ast_v2::bpmn::{convert_bpmn_xml_to_rust_code, convert_rust_code_to_bpmn_xml};

fn usage() -> String {
    "Usage: cargo run --bin bpmn-validator -- <path/to/process.bpmn>".to_string()
}

fn ensure_tmp_dir() -> Result<PathBuf, String> {
    let tmp = PathBuf::from("target").join("tmp");
    fs::create_dir_all(&tmp).map_err(|e| format!("Failed to create {}: {e}", tmp.display()))?;
    Ok(tmp)
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

fn compile_rust_to_exe(src_path: &Path, exe_path: &Path) -> Result<(), String> {
    let output = Command::new("rustc")
        .arg("--edition=2021")
        .arg(src_path)
        .arg("-o")
        .arg(exe_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run rustc: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "rustc failed for {}\n{}",
            src_path.display(),
            stderr
        ));
    }

    Ok(())
}

fn run_exe(exe_path: &Path) -> Result<String, String> {
    let output = Command::new(exe_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run {}: {e}", exe_path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Program exited non-zero: {}\n{}", exe_path.display(), stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
}

fn main() {
    if let Err(e) = real_main() {
        eprintln!("ERROR: {}", e);
        std::process::exit(1);
    }
}

fn real_main() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(bpmn_path) = args.next() else {
        return Err(usage());
    };

    let bpmn_path = PathBuf::from(bpmn_path);
    if !bpmn_path.exists() {
        return Err(format!("Input does not exist: {}", bpmn_path.display()));
    }

    let xml = fs::read_to_string(&bpmn_path)
        .map_err(|e| format!("Failed to read {}: {e}", bpmn_path.display()))?;

    // A) BPMN -> Rust -> run
    let rust_a = convert_bpmn_xml_to_rust_code(&xml)?;

    // B) BPMN -> Rust -> BPMN -> Rust -> run
    let bpmn_roundtrip = convert_rust_code_to_bpmn_xml(&rust_a)?;
    let rust_b = convert_bpmn_xml_to_rust_code(&bpmn_roundtrip)?;

    let tmp = ensure_tmp_dir()?;

    let a_rs = tmp.join("bpmn_validator_a.rs");
    let b_rs = tmp.join("bpmn_validator_b.rs");
    let a_exe = tmp.join(exe_name("bpmn_validator_a"));
    let b_exe = tmp.join(exe_name("bpmn_validator_b"));

    write_file(&a_rs, &rust_a)?;
    write_file(&b_rs, &rust_b)?;

    compile_rust_to_exe(&a_rs, &a_exe)?;
    compile_rust_to_exe(&b_rs, &b_exe)?;

    let out_a = run_exe(&a_exe)?;
    let out_b = run_exe(&b_exe)?;

    if out_a != out_b {
        eprintln!("--- Output A (BPMN->Rust) ---\n{}", out_a);
        eprintln!("--- Output B (BPMN->Rust->BPMN->Rust) ---\n{}", out_b);
        return Err("stdout mismatch between direct and round-tripped pipelines".to_string());
    }

    println!("OK: stdout matches ({} bytes)", out_a.len());
    println!("--- stdout ---\n{}", out_a);

    Ok(())
}
