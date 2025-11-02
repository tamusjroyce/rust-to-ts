use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() {
    // Usage: tester <Examples/HelloWorld>
    let mut args = std::env::args().skip(1);
    let target = args.next().unwrap_or_else(|| {
        eprintln!("Usage: tester <Examples/HelloWorld>");
        std::process::exit(2);
    });

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
    let rust_out = match run_rust_example(&target_path, &rs_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to run Rust example: {}", e);
            std::process::exit(1);
        }
    };

    // 2) Build (if needed) and run the TypeScript example
    let ts_out = match run_ts_example(&ts_file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to run TypeScript example: {}", e);
            std::process::exit(1);
        }
    };

    // 3) Compare outputs exactly
    let matched = rust_out == ts_out;
    if matched {
        println!("Outputs match exactly.");
    } else {
        eprintln!("Outputs differ!");
    }
    // Always show both outputs with consistent labels
    println!("--- Rust ---\n{}", rust_out);
    println!("--- Deno ---\n{}", ts_out);

    std::process::exit(if matched { 0 } else { 1 });
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

fn run_rust_example(example_root: &Path, rs_file: &Path) -> Result<String, String> {
    // If this example is a Cargo project, run it via cargo to resolve dependencies
    let manifest = example_root.join("Cargo.toml");
    if manifest.exists() {
        let cargo_cmd = if cfg!(windows) { "cargo.exe" } else { "cargo" };
        let output = Command::new(cargo_cmd)
            .args(["run", "--quiet", "--manifest-path"])
            .arg(&manifest)
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

    // Compile with rustc into target/tmp/<stem> (platform-specific exe name)
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

fn run_ts_example(ts_file: &Path) -> Result<String, String> {
    // Prefer Deno to execute TypeScript directly. We create a tiny wrapper that
    // appends a call to `main();` so examples with a main() entrypoint run.
    run_with_deno(ts_file)
}

fn run_with_deno(ts_file: &Path) -> Result<String, String> {
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
    let wrapper_code = format!(
        "// auto-generated wrapper for Deno\nimport * as mod from \"{}\";\nasync function run() {{\n  if (typeof (mod as any).main === 'function') {{\n    await (mod as any).main();\n    return;\n  }}\n  // Fallback to global main if someone inlines the function into globalThis\n  const g: any = globalThis as any;\n  if (typeof g.main === 'function') {{\n    g.main();\n    return;\n  }}\n  console.error('No main() found to run');\n  Deno.exit(1);\n}}\nrun().catch((e) => {{ console.error(e); Deno.exit(1); }});\n",
        file_url
    );
    fs::write(&wrapper_path, wrapper_code)
        .map_err(|e| format!("failed to write Deno wrapper: {}", e))?;

    let deno_cmd = if cfg!(windows) { "deno.exe" } else { "deno" };
    let output = Command::new(deno_cmd)
        .arg("run")
        .arg("--quiet")
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
