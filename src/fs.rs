use anyhow::{Context as _, Result};
use glob::{glob_with, MatchOptions};
use std::path::{Path, PathBuf};
use xshell::{cmd, Shell};

pub fn scan<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let pattern = format!("{}/**/*.cr3", path.as_ref().display());
    let results: Result<Vec<PathBuf>, _> = glob_with(&pattern, options)?.collect();
    Ok(results?)
}

pub fn handle_file(
    sh: &Shell,
    source: &Path,
    destination: &Path,
    do_move: bool,
    do_overwrite: bool,
) -> Result<()> {
    let command = match (do_move, do_overwrite) {
        // Move, overwrite
        (true, true) => cmd!(sh, "mv {source} {destination}"),

        // Move, preserve
        (true, false) => cmd!(sh, "mv -n {source} {destination}"),

        // Copy, overwrite
        (false, true) => cmd!(sh, "cp {source} {destination}"),

        // Copy, preserve
        (false, false) => cmd!(sh, "cp -n {source} {destination}"),
    };
    command.run().context("Failed to handle file")
}
