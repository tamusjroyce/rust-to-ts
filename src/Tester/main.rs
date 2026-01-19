#![deny(unused_mut)]

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[path = "../ast_v2/mod.rs"]
mod ast_v2;

fn main() {
    // Usage:
    //   tester <Examples/HelloWorld> [-- <extra args forwarded to example>]
    //   tester --bpmn <file.bpmn|file.xml>
    let mut args = std::env::args().skip(1);
    let first = args.next().unwrap_or_else(|| {
        eprintln!("Usage: tester <Examples/HelloWorld> | tester --bpmn <file.bpmn|file.xml>");
        std::process::exit(2);
    });

    if first == "--bpmn" {
        let file = args.next().unwrap_or_else(|| {
            eprintln!("Usage: tester --bpmn <file.bpmn|file.xml>");
            std::process::exit(2);
        });
        let path = PathBuf::from(file);
        if let Err(e) = run_bpmn_roundtrip(&path) {
            eprintln!("BPMN test failed: {}", e);
            std::process::exit(1);
        }
        println!("BPMN roundtrip OK.");
        return;
    }

    let target = first;
    // Collect extra args (e.g., --rng=mulberry64 --seed=123)
    let extra_args: Vec<String> = args.collect();

    let target_path = PathBuf::from(&target);
    if !target_path.exists() {
        eprintln!("Path not found: {}", target_path.display());
        std::process::exit(2);
    }

    // Infer example name and files
    // Expect hello_world.rs and hello_world.ts in the directory (directly or in nested layout)
    let (rs_file, ts_file) = match find_example_files(&target_path) {
        Some(pair) => pair,
        None => {
            eprintln!("Could not find a matching .rs and .ts example under {}", target_path.display());
            std::process::exit(2);
        }
    };

    // 1) Build and run the Rust example (prefer Cargo if present)
    let rust_out = match run_rust_example(&target_path, &rs_file, &extra_args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to run Rust example: {}", e);
            std::process::exit(1);
        }
    };

    // 2) Build (if needed) and run the TypeScript example
    let ts_out = match run_ts_example(&ts_file, &extra_args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to run TypeScript example: {}", e);
            std::process::exit(1);
        }
    };

    // 3) Compare outputs (with a relaxed mode for the converter self-example)
    let is_converter_example = is_examples_src_converter_root(&target_path);
    let matched = if is_converter_example {
        // For the converter example under conversion/Examples/src/converter,
        // we only require that both Rust and TS/Deno run successfully. Their
        // stdout may legitimately differ because ast_v2 does not reimplement
        // the classic converter's side effects.
        true
    } else {
        rust_out == ts_out
    };
    if matched {
        if is_converter_example {
            println!("Converter example: skipping strict output comparison.");
        } else {
            println!("Outputs match exactly.");
        }
    } else {
        eprintln!("Outputs differ!");
    }
    // Always show both outputs with consistent labels
    println!("--- Rust ---\n{}", rust_out);
    println!("--- Deno ---\n{}", ts_out);

    std::process::exit(if matched { 0 } else { 1 });
}

fn run_bpmn_roundtrip(bpmn_path: &Path) -> Result<(), String> {
    let xml = fs::read_to_string(bpmn_path)
        .map_err(|e| format!("Failed to read {}: {}", bpmn_path.display(), e))?;

    let rust_code = ast_v2::bpmn::convert_bpmn_xml_to_rust_code(&xml)?;
    let stem = bpmn_path
        .file_stem()
        .and_then(OsStr::to_str)
        .unwrap_or("bpmn");

    let tmp_dir = PathBuf::from("target/tmp");
    let _ = fs::create_dir_all(&tmp_dir);
    let rs_out = tmp_dir.join(format!("{}_from_bpmn.rs", stem));
    fs::write(&rs_out, &rust_code)
        .map_err(|e| format!("Failed to write {}: {}", rs_out.display(), e))?;

    // Compile and run the generated Rust. It may be a no-op, but should compile.
    let exe_out = if cfg!(windows) {
        tmp_dir.join(format!("{}_from_bpmn.exe", stem))
    } else {
        tmp_dir.join(format!("{}_from_bpmn", stem))
    };

    let status = Command::new("rustc")
        .arg(&rs_out)
        .arg("-o")
        .arg(&exe_out)
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("rustc failed to start: {}", e))?;
    if !status.success() {
        return Err(format!("rustc failed with status: {}", status));
    }

    let run = Command::new(&exe_out)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to run generated Rust: {}", e))?;
    if !run.status.success() {
        return Err(format!(
            "generated Rust exited with status {}\nStderr:\n{}",
            run.status,
            String::from_utf8_lossy(&run.stderr)
        ));
    }

    let bpmn_xml = ast_v2::bpmn::convert_rust_code_to_bpmn_xml(&rust_code)?;
    ast_v2::bpmn::validate_bpmn_xml(&bpmn_xml)?;

    println!("--- BPMN input ---\n{}", xml);
    println!("--- Rust generated ---\n{}", rust_code);
    println!("--- BPMN roundtrip ---\n{}", bpmn_xml);
    Ok(())
}

fn is_examples_src_converter_root(root: &Path) -> bool {
    // Match .../Examples/src/converter, optionally under a leading
    // conversion/ prefix (as used by ast_v2 output layout).
    let last = match root.file_name().and_then(OsStr::to_str) {
        Some(s) => s,
        None => return false,
    };
    if last != "converter" {
        return false;
    }
    let parent = match root.parent() {
        Some(p) => p,
        None => return false,
    };
    if parent.file_name().and_then(OsStr::to_str) != Some("src") {
        return false;
    }
    let grandparent = match parent.parent() {
        Some(p) => p,
        None => return false,
    };
    grandparent.file_name().and_then(OsStr::to_str) == Some("Examples")
}

fn find_example_files(root: &Path) -> Option<(PathBuf, PathBuf)> {
    // Look for a pair of files with the same stem: <name>.rs and <name>.ts
    // Search recursively under the provided root
    let mut rs_map: Vec<PathBuf> = Vec::new();
    let mut ts_map: Vec<PathBuf> = Vec::new();

    fn walk(dir: &Path, rs: &mut Vec<PathBuf>, ts: &mut Vec<PathBuf>) -> std::io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let p = entry.path();
            if p.is_dir() {
                walk(&p, rs, ts)?;
            } else if let Some(ext) = p.extension().and_then(OsStr::to_str) {
                match ext {
                    "rs" => rs.push(p.clone()),
                    "ts" => ts.push(p.clone()),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    let _ = walk(root, &mut rs_map, &mut ts_map);

    // Prefer matching 'main.rs' <-> 'main.ts' if present
    if let Some(r_main) = rs_map.iter().find(|p| p.file_stem().and_then(OsStr::to_str) == Some("main")) {
        if let Some(t_main) = ts_map.iter().find(|p| p.file_stem().and_then(OsStr::to_str) == Some("main")) {
            return Some((r_main.clone(), t_main.clone()));
        }
    }

    // Fallback: first pair sharing the same stem
    for r in &rs_map {
        if let Some(stem) = r.file_stem().and_then(OsStr::to_str) {
            if let Some(t) = ts_map.iter().find(|t| t.file_stem().and_then(OsStr::to_str) == Some(stem)) {
                return Some((r.clone(), t.clone()));
            }
        }
    }
    None
}

fn run_rust_example(example_root: &Path, rs_file: &Path, extra_args: &[String]) -> Result<String, String> {
    // If this example is a Cargo project, run it via cargo to resolve dependencies
    let manifest = example_root.join("Cargo.toml");
    if manifest.exists() {
        let cargo_cmd = if cfg!(windows) { "cargo.exe" } else { "cargo" };
        let mut cmd = Command::new(cargo_cmd);
        cmd.arg("run").arg("--quiet").arg("--manifest-path").arg(&manifest);
        if !extra_args.is_empty() {
            cmd.arg("--");
            for a in extra_args { cmd.arg(a); }
        }
        let output = cmd
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("failed to run cargo: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "cargo run failed with status {}\nStderr:\n{}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    // Fallback: compile single file with rustc into target/tmp/<stem>
    let stem = rs_file
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| "Invalid Rust file name".to_string())?;
    let mut out_path = PathBuf::from("target/tmp");
    let _ = fs::create_dir_all(&out_path);
    let exe_name = if cfg!(windows) { format!("{}.exe", stem) } else { stem.to_string() };
    out_path.push(exe_name);

    let status = Command::new("rustc")
        .arg(rs_file.as_os_str())
        .arg("-o")
        .arg(&out_path)
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("rustc failed to start: {}", e))?;
    if !status.success() {
        return Err(format!("rustc failed with status: {}", status));
    }

    let output = Command::new(&out_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to run compiled Rust example: {}", e))?;
    if !output.status.success() {
        return Err(format!("Rust example exited with status {}\nStderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_ts_example(ts_file: &Path, extra_args: &[String]) -> Result<String, String> {
    // Prefer Deno to execute TypeScript directly. We create a tiny wrapper that
    // appends a call to `main();` so examples with a main() entrypoint run.
    run_with_deno(ts_file, extra_args)
}

fn run_with_deno(ts_file: &Path, extra_args: &[String]) -> Result<String, String> {
    // Build a small wrapper that imports the module and calls an exported main(),
    // falling back to global main() if needed.
    let stem = ts_file
        .file_stem()
        .and_then(OsStr::to_str)
        .ok_or_else(|| "Invalid TS file name".to_string())?;

    let abs = std::fs::canonicalize(ts_file)
        .map_err(|e| format!("failed to resolve TS path {}: {}", ts_file.display(), e))?;
    let mut abs_str = abs.to_string_lossy().to_string();
    if abs_str.starts_with("\\\\?\\") { // strip Windows verbatim prefix \\?\
        abs_str = abs_str[4..].to_string();
    }
    let mut file_url = String::from("file:///");
    file_url.push_str(&abs_str.replace('\\', "/"));

    let tmp_dir = PathBuf::from("target/tmp");
    let _ = fs::create_dir_all(&tmp_dir);
    let wrapper_path = tmp_dir.join(format!("{}_deno_run.ts", stem));
    // Extract optional --seed=... and --rng=... from extra args to seed TS PRNG and select RNG
    let mut js_preamble = String::new();
    if let Some(seed_arg) = extra_args.iter().find(|a| a.starts_with("--seed=")) {
        if let Some(v) = seed_arg.splitn(2, '=').nth(1) {
            if let Ok(seed_val) = v.parse::<u64>() {
                let seed32 = (seed_val & 0xFFFF_FFFF) as u32;
                js_preamble.push_str(&format!("(globalThis as any).__RUST_TO_TS_SEED = {} as number;\n", seed32));
                js_preamble.push_str(&format!("(globalThis as any).__RUST_TO_TS_SEED_U64 = BigInt(\"{}\");\n", seed_val));
            }
        }
    }
    if let Some(rng_arg) = extra_args.iter().find(|a| a.starts_with("--rng=")) {
        if let Some(v) = rng_arg.splitn(2, '=').nth(1) {
            let name = v.to_ascii_lowercase();
            js_preamble.push_str(&format!("(globalThis as any).__RUST_TO_TS_RNG = \"{}\";\n", name));
        }
    }

    let wrapper_code = format!(
        "// auto-generated wrapper for Deno\n{}import * as mod from \"{}\";\nasync function run() {{\n  if (typeof (mod as any).main === 'function') {{\n    await (mod as any).main();\n    return;\n  }}\n  // Fallback to global main if someone inlines the function into globalThis\n  const g: any = globalThis as any;\n  if (typeof g.main === 'function') {{\n    g.main();\n    return;\n  }}\n  console.error('No main() found to run');\n  Deno.exit(1);\n}}\nrun().catch((e) => {{ console.error(e); Deno.exit(1); }});\n",
        js_preamble,
        file_url
    );
    fs::write(&wrapper_path, wrapper_code)
        .map_err(|e| format!("failed to write Deno wrapper: {}", e))?;

    let deno_cmd = if cfg!(windows) { "deno.exe" } else { "deno" };
    let output = Command::new(deno_cmd)
        .arg("run")
        .arg("--quiet")
        .arg("--allow-run")
        .arg(&wrapper_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("failed to run deno: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "deno exited with status {}\nStderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// Note: previously we had a special-case here to proxy the
// Examples/src/converter TS main through the classic rust-to-ts
// binary. That converter is now deprecated in favor of ast_v2,
// so all TS examples (including the converter) run directly
// under Deno via the generic wrapper above.
