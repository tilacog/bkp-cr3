mod cli;
mod exif;
mod fs;

use crate::exif::Metadata;
use anyhow::Context as _;
use bloom::{BloomFilter, ASMS};
use clap::Parser as _;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use xshell::{cmd, Shell};

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let sh = xshell::Shell::new()?;
    let mut runner = Runner::new(
        sh,
        &args.source,
        &args.destination,
        args.dry_run,
        args.overwrite,
        args.do_move,
    )?;
    runner.run()
}

struct Runner<'a> {
    filter: BloomFilter,
    sh: xshell::Shell,
    input_files: Vec<PathBuf>,
    destination: &'a Path,
    dry_run: bool,
    overwrite: bool,
    do_move: bool,
}

impl<'a> Runner<'a> {
    fn new(
        sh: Shell,
        source: &Path,
        destination: &'a Path,
        dry_run: bool,
        overwrite: bool,
        do_move: bool,
    ) -> anyhow::Result<Self> {
        let input_files = fs::scan(source)?;
        let filter = BloomFilter::with_rate(0.01, input_files.len() as u32);
        Ok(Self {
            filter,
            sh,
            input_files,
            destination,
            dry_run,
            overwrite,
            do_move,
        })
    }
    fn run(&mut self) -> anyhow::Result<()> {
        let mut duplicates: HashMap<PathBuf, HashSet<&Path>> = HashMap::new();

        for file_name in self.input_files.iter() {
            let metadata = Metadata::exiftool(&self.sh, &file_name)?;
            let new_file_name = metadata.new_file_name(self.destination);
            if self.filter.contains(&new_file_name) {
                // register duplicate
                duplicates
                    .entry(new_file_name)
                    .or_default()
                    .insert(file_name);
            } else {
                self.handle_file(file_name, &new_file_name)?;
                self.filter.insert(&new_file_name);
            }
        }
        self.handle_duplicates(duplicates)
    }

    fn handle_file(&self, old: &Path, new: &Path) -> anyhow::Result<()> {
        if self.dry_run {
            println!("{} -> {}", old.display(), new.display());
            return Ok(());
        }
        let command = match (self.do_move, self.overwrite) {
            // Move, overwrite
            (true, true) => cmd!(self.sh, "mv {old} {new}"),

            // Move, preserve
            (true, false) => cmd!(self.sh, "mv -n {old} {new}"),

            // Copy, overwrite
            (false, true) => cmd!(self.sh, "cp {old} {new}"),

            // Copy, preserve
            (false, false) => cmd!(self.sh, "cp -n {old} {new}"),
        };

        command.run().context("Failed to handle file")
    }

    fn handle_duplicates(
        &self,
        duplicates: HashMap<PathBuf, HashSet<&Path>>,
    ) -> anyhow::Result<()> {
        for (new_file_name, old_file_names) in duplicates.into_iter() {
            if old_file_names.len() == 1 {
                // Handle false positives
                let old_file_name = old_file_names.into_iter().next().unwrap();
                self.handle_file(old_file_name, &new_file_name)?;
                return Ok(());
            } else {
                let mut counter: u32 = 1;
                for old_file_name in old_file_names {
                    let new_file_name_with_suffix = increment_name(&new_file_name, counter);
                    self.handle_file(&new_file_name_with_suffix, old_file_name)?;
                    counter += 1
                }
            }
        }
        Ok(())
    }
}

fn increment_name(input: &Path, number: u32) -> PathBuf {
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
