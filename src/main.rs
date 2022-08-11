mod cli;
mod exif;
mod fs;

use crate::exif::Metadata;
use bloom::{BloomFilter, ASMS};
use clap::Parser as _;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use xshell::Shell;

type Duplicates<'a> = HashMap<PathBuf, HashSet<&'a Path>>;
type Uniques<'a> = HashMap<PathBuf, &'a Path>;

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
        Ok(Self {
            sh,
            input_files,
            destination,
            dry_run,
            overwrite,
            do_move,
        })
    }

    fn run(&mut self) -> anyhow::Result<()> {
        let mut filter = BloomFilter::with_rate(0.01, self.input_files.len() as u32);
        let mut duplicates: Duplicates = HashMap::new();
        let mut uniques: Uniques = HashMap::new();

        for source_file_name in self.input_files.iter() {
            let metadata = Metadata::exiftool(&self.sh, &source_file_name)?;
            let dest_file_name = metadata.new_file_name(self.destination);

            if filter.contains(&dest_file_name) {
                // If the filter has seen this name before, then this is probably a duplicate.
                // We need to:
                // 1. remove the previous ocurrence from the `uniques` container, if it is there.
                // 2. if the previous operation succeeded, insert the current file name in the
                //   `duplicates` container.
                if let Some(prev_source_file_name) = uniques.remove(&dest_file_name) {
                    insert_duplicate(&mut duplicates, dest_file_name.clone(), source_file_name);
                    insert_duplicate(&mut duplicates, dest_file_name, prev_source_file_name);
                    continue;
                }
            }
            insert_unique(&mut filter, &mut uniques, dest_file_name, &source_file_name);
        }
        self.handle_uniques(uniques)?;
        self.handle_duplicates(duplicates)
    }

    fn handle_file(&self, source: &Path, destination: &Path) -> anyhow::Result<()> {
        if self.dry_run {
            println!("{} -> {}", source.display(), destination.display());
            Ok(())
        } else {
            fs::handle_file(&self.sh, source, destination, self.do_move, self.overwrite)
        }
    }

    fn handle_uniques(&self, uniques: HashMap<PathBuf, &Path>) -> anyhow::Result<()> {
        for (dest_file_name, source_file_name) in uniques.into_iter() {
            self.handle_file(source_file_name, &dest_file_name)?;
        }
        Ok(())
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
                    self.handle_file(old_file_name, &new_file_name_with_suffix)?;
                    counter += 1
                }
            }
        }
        Ok(())
    }
}

fn insert_duplicate<'a>(duplicates: &mut Duplicates<'a>, key: PathBuf, value: &'a Path) {
    assert!(duplicates.entry(key).or_default().insert(value));
}

fn insert_unique<'a>(
    filter: &mut BloomFilter,
    uniques: &mut Uniques<'a>,
    key: PathBuf,
    value: &'a Path,
) {
    filter.insert(&key);
    assert!(uniques.insert(key, value).is_none());
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
