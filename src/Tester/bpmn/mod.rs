use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::ast_v2;

pub fn run_bpmn_roundtrip(input_path: &Path) -> Result<(), String> {
    if !input_path.exists() {
        return Err(format!("Input does not exist: {}", input_path.display()));
    }

    if input_path.is_dir() {
        return validate_rust_dir_roundtrip(input_path);
    }

    let ext = input_path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "bpmn" | "xml" => validate_bpmn_file_roundtrip(input_path),
        "rs" => validate_rust_file_roundtrip(input_path),
        _ => Err(format!(
            "Unsupported --bpmn input (expected .bpmn/.xml/.rs or dir): {}",
            input_path.display()
        )),
    }
}

fn ensure_tmp_dir() -> Result<PathBuf, String> {
    let tmp = PathBuf::from("target").join("tmp");
    fs::create_dir_all(&tmp).map_err(|e| format!("Failed to create {}: {e}", tmp.display()))?;
    Ok(tmp)
}

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
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
        return Err(format!("rustc failed for {}\n{}", src_path.display(), stderr));
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
        return Err(format!(
            "Program exited non-zero: {}\n{}",
            exe_path.display(),
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn validate_bpmn_file_roundtrip(bpmn_path: &Path) -> Result<(), String> {
    let xml = fs::read_to_string(bpmn_path)
        .map_err(|e| format!("Failed to read {}: {e}", bpmn_path.display()))?;

    // A) BPMN -> Rust (prefers embedded rustSource when present)
    let rust_a = ast_v2::bpmn::convert_bpmn_xml_to_rust_code(&xml)?;

    // B) Rust -> BPMN -> Rust
    let bpmn_roundtrip = ast_v2::bpmn::convert_rust_code_to_bpmn_xml(&rust_a)?;
    ast_v2::bpmn::validate_bpmn_xml(&bpmn_roundtrip)?;
    let rust_b = ast_v2::bpmn::convert_bpmn_xml_to_rust_code(&bpmn_roundtrip)?;

    // Compile and run both, then compare stdout (this matches the old bpmn-validator semantics).
    let tmp = ensure_tmp_dir()?;
    let stem = bpmn_path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("bpmn");

    let a_rs = tmp.join(format!("{}_bpmn_a.rs", stem));
    let b_rs = tmp.join(format!("{}_bpmn_b.rs", stem));
    let a_exe = tmp.join(exe_name(&format!("{}_bpmn_a", stem)));
    let b_exe = tmp.join(exe_name(&format!("{}_bpmn_b", stem)));

    fs::write(&a_rs, &rust_a).map_err(|e| format!("Failed to write {}: {e}", a_rs.display()))?;
    fs::write(&b_rs, &rust_b).map_err(|e| format!("Failed to write {}: {e}", b_rs.display()))?;
    compile_rust_to_exe(&a_rs, &a_exe)?;
    compile_rust_to_exe(&b_rs, &b_exe)?;

    let out_a = run_exe(&a_exe)?;
    let out_b = run_exe(&b_exe)?;
    if out_a != out_b {
        eprintln!("--- Output A (BPMN->Rust) ---\n{}", out_a);
        eprintln!("--- Output B (BPMN->Rust->BPMN->Rust) ---\n{}", out_b);
        return Err("stdout mismatch between direct and round-tripped pipelines".to_string());
    }

    Ok(())
}

fn validate_rust_file_roundtrip(rs_path: &Path) -> Result<(), String> {
    let rust_src = fs::read_to_string(rs_path)
        .map_err(|e| format!("Failed to read {}: {e}", rs_path.display()))?;

    let bpmn = ast_v2::bpmn::convert_rust_code_to_bpmn_xml(&rust_src)?;
    ast_v2::bpmn::validate_bpmn_xml(&bpmn)?;
    let rust_roundtrip = ast_v2::bpmn::convert_bpmn_xml_to_rust_code(&bpmn)?;

    if rust_src != rust_roundtrip {
        return Err("rust source mismatch after rust->bpmn->rust".to_string());
    }

    // Also compile+run to ensure the code is runnable as a standalone file.
    let tmp = ensure_tmp_dir()?;
    let stem = rs_path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("input");

    let a_rs = tmp.join(format!("{}_rust_a.rs", stem));
    let b_rs = tmp.join(format!("{}_rust_b.rs", stem));
    let a_exe = tmp.join(exe_name(&format!("{}_rust_a", stem)));
    let b_exe = tmp.join(exe_name(&format!("{}_rust_b", stem)));

    fs::write(&a_rs, &rust_src).map_err(|e| format!("Failed to write {}: {e}", a_rs.display()))?;
    fs::write(&b_rs, &rust_roundtrip)
        .map_err(|e| format!("Failed to write {}: {e}", b_rs.display()))?;
    compile_rust_to_exe(&a_rs, &a_exe)?;
    compile_rust_to_exe(&b_rs, &b_exe)?;

    let out_a = run_exe(&a_exe)?;
    let out_b = run_exe(&b_exe)?;
    if out_a != out_b {
        return Err("stdout mismatch after rust->bpmn->rust".to_string());
    }

    let out_bpmn = tmp.join("tester_from_rust.bpmn");
    fs::write(&out_bpmn, &bpmn)
        .map_err(|e| format!("Failed to write {}: {e}", out_bpmn.display()))?;
    Ok(())
}

fn validate_rust_dir_roundtrip(root: &Path) -> Result<(), String> {
    let bpmn = ast_v2::bpmn::convert_rust_dir_to_bpmn_xml(root)?;
    ast_v2::bpmn::validate_bpmn_xml(&bpmn)?;
    let sources = ast_v2::bpmn::convert_bpmn_xml_to_rust_sources(&bpmn)?;
    if sources.is_empty() {
        return Err("no rustSource nodes extracted from generated BPMN".to_string());
    }

    let tmp = ensure_tmp_dir()?;
    let out_bpmn = tmp.join("tester_from_rust_dir.bpmn");
    fs::write(&out_bpmn, &bpmn)
        .map_err(|e| format!("Failed to write {}: {e}", out_bpmn.display()))?;

    let out_root = tmp.join("tester_bpmn_roundtrip");
    fs::create_dir_all(&out_root)
        .map_err(|e| format!("Failed to create {}: {e}", out_root.display()))?;

    let mut mismatches: Vec<String> = Vec::new();
    for (path_opt, src) in sources {
        let Some(rel) = path_opt else {
            mismatches.push("missing path attribute on rustSource".to_string());
            continue;
        };

        let rel_path = PathBuf::from(&rel);
        let original_path = root.join(&rel_path);
        let recovered_path = out_root.join(&rel_path);

        if let Some(parent) = recovered_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {e}", parent.display()))?;
        }
        fs::write(&recovered_path, &src)
            .map_err(|e| format!("Failed to write {}: {e}", recovered_path.display()))?;

        let orig = fs::read_to_string(&original_path)
            .map_err(|e| format!("Failed to read original {}: {e}", original_path.display()))?;
        if orig != src {
            mismatches.push(rel);
        }
    }

    if !mismatches.is_empty() {
        return Err(format!(
            "{} file(s) mismatched after rust-dir->bpmn->rust roundtrip: {}",
            mismatches.len(),
            mismatches.join(", ")
        ));
    }

    Ok(())
}
