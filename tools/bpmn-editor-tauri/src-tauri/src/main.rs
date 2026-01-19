#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde::Serialize;

use rust_to_ts::ast_v2::bpmn::{
    convert_bpmn_xml_to_module, convert_bpmn_xml_to_rust_code, convert_rust_code_to_bpmn_xml,
};
use rust_to_ts::ast_v2::{module_to_ts};

#[derive(Debug, Serialize)]
struct ValidateResult {
    ok: bool,
    stdout_direct: String,
    stdout_roundtrip: String,
    rust_direct: String,
    rust_roundtrip: String,
    bpmn_roundtrip: String,
}

fn tmp_dir() -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("rust-to-ts-bpmn-editor");
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create temp dir {}: {e}", dir.display()))?;
    Ok(dir)
}

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
}

fn compile_rust_to_exe(src_path: &PathBuf, exe_path: &PathBuf) -> Result<(), String> {
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
        return Err(format!(
            "rustc failed\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

fn run_exe(exe_path: &PathBuf) -> Result<String, String> {
    let output = Command::new(exe_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run {}: {e}", exe_path.display()))?;

    if !output.status.success() {
        return Err(format!(
            "Program exited non-zero\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {e}", path))
}

#[tauri::command]
fn write_text_file(path: String, contents: String) -> Result<(), String> {
    fs::write(&path, contents).map_err(|e| format!("Failed to write {}: {e}", path))
}

#[tauri::command]
fn bpmn_to_rust(xml: String) -> Result<String, String> {
    convert_bpmn_xml_to_rust_code(&xml)
}

#[tauri::command]
fn bpmn_to_ts(xml: String) -> Result<String, String> {
    let module = convert_bpmn_xml_to_module(&xml)?;
    Ok(module_to_ts(&module))
}

#[tauri::command]
fn validate_roundtrip(xml: String) -> Result<ValidateResult, String> {
    // Direct
    let rust_direct = convert_bpmn_xml_to_rust_code(&xml)?;

    // Round-trip through BPMN
    let bpmn_roundtrip = convert_rust_code_to_bpmn_xml(&rust_direct)?;
    let rust_roundtrip = convert_bpmn_xml_to_rust_code(&bpmn_roundtrip)?;

    let tmp = tmp_dir()?;
    let direct_rs = tmp.join("direct.rs");
    let round_rs = tmp.join("roundtrip.rs");
    let direct_exe = tmp.join(exe_name("direct"));
    let round_exe = tmp.join(exe_name("roundtrip"));

    fs::write(&direct_rs, &rust_direct)
        .map_err(|e| format!("Failed to write {}: {e}", direct_rs.display()))?;
    fs::write(&round_rs, &rust_roundtrip)
        .map_err(|e| format!("Failed to write {}: {e}", round_rs.display()))?;

    compile_rust_to_exe(&direct_rs, &direct_exe)?;
    compile_rust_to_exe(&round_rs, &round_exe)?;

    let stdout_direct = run_exe(&direct_exe)?;
    let stdout_roundtrip = run_exe(&round_exe)?;
    let ok = stdout_direct == stdout_roundtrip;

    Ok(ValidateResult {
        ok,
        stdout_direct,
        stdout_roundtrip,
        rust_direct,
        rust_roundtrip,
        bpmn_roundtrip,
    })
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            read_text_file,
            write_text_file,
            bpmn_to_rust,
            bpmn_to_ts,
            validate_roundtrip,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
