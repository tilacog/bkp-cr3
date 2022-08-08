use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(value_parser = validate_directory)]
    pub source: PathBuf,

    #[clap(value_parser = validate_directory)]
    pub destination: PathBuf,

    #[clap(value_parser, short, long, action)]
    /// Don't perform any write operation
    pub dry_run: bool,

    #[clap(value_parser, short, long, action)]
    /// Overwrite TARGET, if it exists
    pub overwrite: bool,
}

fn validate_directory(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.exists() {
        Err(format!("Directory does not exist: {path:?}"))
    } else if !path.is_dir() {
        Err(format!("Path is not a directory: {path:?}"))
    } else {
        Ok(path)
    }
}
