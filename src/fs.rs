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

pub fn increment_name(input: &Path, number: u32) -> PathBuf {
    let extension = input.extension().expect("Expected an extension");
    let base_name = input.with_extension("");
    let base_name = base_name.to_str().expect("Expected a base name");
    let new_name = format!("{}-{:0>3}", base_name, number);
    input.with_file_name(&new_name).with_extension(extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_name() {
        let path = Path::new("/etc/foo/bar.rs");
        let new = increment_name(path, 1);
        assert_eq!(new, Path::new("/etc/foo/bar-001.rs"))
    }
}
