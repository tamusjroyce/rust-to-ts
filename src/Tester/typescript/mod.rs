use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn run_ts_example(ts_file: &Path, extra_args: &[String]) -> Result<String, String> {
    // Prefer Deno to execute TypeScript directly. We create a tiny wrapper that
    // imports the module and calls an exported main(), falling back to global main().
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
    if abs_str.starts_with("\\\\?\\") {
        // strip Windows verbatim prefix \\?\
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
                js_preamble.push_str(&format!(
                    "(globalThis as any).__RUST_TO_TS_SEED = {} as number;\n",
                    seed32
                ));
                js_preamble.push_str(&format!(
                    "(globalThis as any).__RUST_TO_TS_SEED_U64 = BigInt(\"{}\");\n",
                    seed_val
                ));
            }
        }
    }
    if let Some(rng_arg) = extra_args.iter().find(|a| a.starts_with("--rng=")) {
        if let Some(v) = rng_arg.splitn(2, '=').nth(1) {
            let name = v.to_ascii_lowercase();
            js_preamble.push_str(&format!(
                "(globalThis as any).__RUST_TO_TS_RNG = \"{}\";\n",
                name
            ));
        }
    }

    let wrapper_code = format!(
        "// auto-generated wrapper for Deno\n{}import * as mod from \"{}\";\nasync function run() {{\n  if (typeof (mod as any).main === 'function') {{\n    await (mod as any).main();\n    return;\n  }}\n  // Fallback to global main if someone inlines the function into globalThis\n  const g: any = globalThis as any;\n  if (typeof g.main === 'function') {{\n    g.main();\n    return;\n  }}\n  console.error('No main() found to run');\n  Deno.exit(1);\n}}\nrun().catch((e) => {{ console.error(e); Deno.exit(1); }});\n",
        js_preamble, file_url
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
