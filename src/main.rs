use std::env;
use std::path::PathBuf;

mod converter;

fn main() {
    // If no args, default to ./Examples
    let args: Vec<String> = env::args().skip(1).collect();
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