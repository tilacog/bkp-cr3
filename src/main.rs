mod cli;
mod exif;
mod fs;

use crate::exif::Metadata;
use clap::Parser as _;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use xshell::Shell;

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
        let mut files: HashMap<&Path, HashSet<PathBuf>> = HashMap::new();

        // parse metadata and group
        for source in self.input_files.iter() {
            let metadata = Metadata::exiftool(&self.sh, &source)?;
            let dest = metadata.new_file_name(self.destination);
            files.entry(source).or_default().insert(dest);
        }

        // fan out
        for (destination, sources) in files.into_iter() {
            if sources.len() == 1 {
                let source = sources.into_iter().next().unwrap();
                self.handle_file(&source, destination)?;
            } else {
                self.handle_duplicates(destination, sources)?
            }
        }
        Ok(())
    }

    fn handle_file(&self, source: &Path, destination: &Path) -> anyhow::Result<()> {
        if self.dry_run {
            println!("{} -> {}", source.display(), destination.display());
            Ok(())
        } else {
            fs::handle_file(&self.sh, source, destination, self.do_move, self.overwrite)
        }
    }

    fn handle_duplicates(
        &self,
        destination_prefix: &Path,
        sources: HashSet<PathBuf>,
    ) -> anyhow::Result<()> {
        let mut counter: u32 = 1;
        for old in sources.into_iter() {
            let destination_with_suffix = increment_name(destination_prefix, counter);
            self.handle_file(&old, &destination_with_suffix)?;
            counter += 1
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
