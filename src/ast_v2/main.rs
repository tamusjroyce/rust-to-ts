#![deny(unused_mut)]
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "mod.rs"]
mod ast_v2;

use ast_v2::{convert_rust_file_to_ts, convert_ts_file_to_rust, module_to_rust, module_to_ts};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    RsToTs,
    TsToRs,
    Both,
}

fn main() {
    let mut args = env::args().skip(1);
    let mut dry_run = false;
    let mut output_dir: Option<PathBuf> = None;
    let mut inputs: Vec<PathBuf> = Vec::new();
    let mut direction = Direction::RsToTs;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--output-dir" => {
                let val = args.next().unwrap_or_else(|| {
                    eprintln!("--output-dir requires a path argument");
                    std::process::exit(1);
                });
                output_dir = Some(PathBuf::from(val));
            }
            "--ts-to-rs" => direction = Direction::TsToRs,
            "--both" => direction = Direction::Both,
            _ => inputs.push(PathBuf::from(arg)),
        }
    }

    if inputs.is_empty() {
        eprintln!("Usage: ast-v2 [--dry-run] [--output-dir DIR] <path> [more paths...]");
        eprintln!("  <path> can be a file or directory, e.g. Examples/HelloWorld");
        std::process::exit(2);
    }

    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("conversion"));

    if !dry_run {
        if let Err(e) = fs::create_dir_all(&output_dir) {
            eprintln!("Failed to create output dir {}: {}", output_dir.display(), e);
            std::process::exit(1);
        }
    }

    let mut had_error = false;

    for input in &inputs {
        if let Err(e) = process_path(input, &output_dir, dry_run, direction) {
            eprintln!("{}", e);
            had_error = true;
        }
    }

    if had_error {
        std::process::exit(1);
    }
}

fn process_path(path: &Path, output_dir: &Path, dry_run: bool, direction: Direction) -> Result<(), String> {
    if path.starts_with(output_dir) {
        return Ok(());
    }

    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to stat {}: {}", path.display(), e))?;

    if metadata.is_dir() {
        if path == output_dir {
            return Ok(());
        }
        for entry in fs::read_dir(path)
            .map_err(|e| format!("Failed to read dir {}: {}", path.display(), e))?
        {
            let entry = entry
                .map_err(|e| format!("Failed to read dir entry in {}: {}", path.display(), e))?;
            let child = entry.path();
            process_path(&child, output_dir, dry_run, direction)?;
        }
        return Ok(());
    }

    // Mirror Cargo.toml so tester can use Cargo-based execution for
    // examples like NeuralNetwork under the conversion tree.
    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
        if file_name == "Cargo.toml" {
            let rel = if path.is_absolute() {
                let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                path
                    .strip_prefix(&cwd)
                    .unwrap_or(path)
                    .to_path_buf()
            } else {
                path.to_path_buf()
            };

            let dest = output_dir.join(&rel);
            if dry_run {
                println!(
                    "--dry-run: would mirror Cargo.toml {} -> {}",
                    path.display(),
                    dest.display()
                );
                return Ok(());
            }

            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
            }
            fs::copy(path, &dest).map_err(|e| {
                format!("Failed to copy Cargo.toml {} -> {}: {}", path.display(), dest.display(), e)
            })?;

            return Ok(());
        }
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext != "rs" && ext != "ts" && ext != "bpmn" {
        return Ok(());
    }

    // Preserve original directory structure under the output directory.
    let rel = if path.is_absolute() {
        let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        path
            .strip_prefix(&cwd)
            .unwrap_or(path)
            .to_path_buf()
    } else {
        path.to_path_buf()
    };

    // Always mirror the original source file into the output tree so that
    // validation tools can operate purely within `output_dir`.
    let src_mirror_path = output_dir.join(&rel);

    if dry_run {
        match ext.as_str() {
            "bpmn" => {
                let mut rs_dest = rel.clone();
                rs_dest.set_extension("rs");
                let mut ts_dest = rel.clone();
                ts_dest.set_extension("ts");
                println!(
                    "--dry-run: would mirror {} into {}",
                    path.display(),
                    src_mirror_path.display()
                );
                println!(
                    "--dry-run: would convert {} -> {} and {}",
                    path.display(),
                    output_dir.join(&rs_dest).display(),
                    output_dir.join(&ts_dest).display()
                );
                return Ok(());
            }
            _ => {
                // fall through to existing rs/ts dry-run below
            }
        }
    }

    let mut dest_rel = rel.clone();
    dest_rel.set_extension(if ext == "rs" { "ts" } else { "rs" });
    let dest_path = output_dir.join(&dest_rel);

    if dry_run {
        match (ext.as_str(), direction) {
            ("rs", Direction::RsToTs) | ("rs", Direction::Both) => {
                println!(
                    "--dry-run: would convert {} -> {}",
                    path.display(),
                    dest_path.display()
                );
            }
            ("ts", Direction::TsToRs) | ("ts", Direction::Both) => {
                println!(
                    "--dry-run: would convert {} -> {}",
                    path.display(),
                    dest_path.display()
                );
            }
            _ => {
                println!(
                    "--dry-run: would mirror source {} into {}",
                    path.display(),
                    src_mirror_path.display()
                );
            }
        }
        return Ok(());
    }

    if let Some(parent) = src_mirror_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
    }
    fs::copy(path, &src_mirror_path)
        .map_err(|e| format!("Failed to copy {} -> {}: {}", path.display(), src_mirror_path.display(), e))?;

    if ext == "bpmn" {
        let xml = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        let module = ast_v2::bpmn::convert_bpmn_xml_to_module(&xml)?;

        let mut rs_dest_rel = rel.clone();
        rs_dest_rel.set_extension("rs");
        let rs_dest_path = output_dir.join(&rs_dest_rel);
        if let Some(parent) = rs_dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
        }
        fs::write(&rs_dest_path, module_to_rust(&module))
            .map_err(|e| format!("Failed to write {}: {}", rs_dest_path.display(), e))?;
        println!("Wrote {}", rs_dest_path.display());

        let mut ts_dest_rel = rel.clone();
        ts_dest_rel.set_extension("ts");
        let ts_dest_path = output_dir.join(&ts_dest_rel);
        if let Some(parent) = ts_dest_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
        }
        fs::write(&ts_dest_path, module_to_ts(&module))
            .map_err(|e| format!("Failed to write {}: {}", ts_dest_path.display(), e))?;
        println!("Wrote {}", ts_dest_path.display());

        return Ok(());
    }

    match (ext.as_str(), direction) {
        ("rs", Direction::RsToTs) | ("rs", Direction::Both) => {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
            }

            let converted = convert_rust_file_to_ts(path)?;

            fs::write(&dest_path, converted)
                .map_err(|e| format!("Failed to write {}: {}", dest_path.display(), e))?;
            println!("Wrote {}", dest_path.display());
        }
        ("ts", Direction::TsToRs) | ("ts", Direction::Both) => {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
            }

            let converted = convert_ts_file_to_rust(path)?;

            fs::write(&dest_path, converted)
                .map_err(|e| format!("Failed to write {}: {}", dest_path.display(), e))?;
            println!("Wrote {}", dest_path.display());
        }
        _ => {
            // Direction doesn't request a conversion for this extension;
            // we already mirrored the source above.
        }
    }

    Ok(())
}
