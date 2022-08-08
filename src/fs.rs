use anyhow::Result;
use glob::glob_with;
use glob::MatchOptions;
use std::path::{Path, PathBuf};

pub fn scan<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let pattern = format!("{}/*.cr3", path.as_ref().display());
    let results: Result<Vec<PathBuf>, _> = glob_with(&pattern, options)?.collect();
    Ok(results?)
}
