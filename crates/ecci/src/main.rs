use ecci::selection::{select_paths, Outcome};
use std::path::PathBuf;

fn main() {
    let mut paths: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if paths.is_empty() {
        paths.push(PathBuf::from("."));
    }

    let selection = select_paths(&paths);
    for file in &selection.files {
        println!("{}", file.path.display());
    }
    let mut failed = false;
    for outcome in &selection.outcomes {
        if let Outcome::Error {
            path,
            operation,
            detail,
            ..
        } = outcome
        {
            failed = true;
            eprintln!(
                "error[ECCI-IO] {}: {}: {}",
                path.display(),
                operation,
                detail
            );
        }
    }
    if failed {
        std::process::exit(3);
    }
}
