#![deny(unused_mut)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[path = "mod.rs"]
mod ast_v2;

fn main() {
    let mut args = env::args().skip(1);
    let mut dry_run = false;
    let mut output_dir: Option<PathBuf> = None;
    let mut inputs: Vec<PathBuf> = Vec::new();

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
        if let Err(e) = process_path(input, &output_dir, dry_run) {
            eprintln!("{}", e);
            had_error = true;
        }
    }

    if had_error {
        std::process::exit(1);
    }
}

fn process_path(path: &Path, output_dir: &Path, dry_run: bool) -> Result<(), String> {
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
            process_path(&child, output_dir, dry_run)?;
        }
        return Ok(());
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    if ext != "rs" && ext != "ts" {
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

    let mut dest_rel = rel;
    dest_rel.set_extension(if ext == "rs" { "ts" } else { "rs" });
    let dest_path = output_dir.join(dest_rel);

    if dry_run {
        println!(
            "--dry-run: would convert {} -> {}",
            path.display(),
            dest_path.display()
        );
        return Ok(());
    }

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
    }

    let converted = if ext == "rs" {
        ast_v2::convert_rust_file_to_ts(path)
    } else {
        ast_v2::convert_ts_file_to_rust(path)
    }?;

    fs::write(&dest_path, converted)
        .map_err(|e| format!("Failed to write {}: {}", dest_path.display(), e))?;
    println!("Wrote {}", dest_path.display());
    Ok(())
}
