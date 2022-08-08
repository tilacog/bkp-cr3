mod cli;
mod exif;
mod fs;

use bloom::{BloomFilter, ASMS};
use clap::Parser as _;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};
use xshell::Shell;

use crate::exif::Metadata;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let sh = xshell::Shell::new()?;
    let mut runner = Runner::new(
        sh,
        &args.source,
        &args.destination,
        args.dry_run,
        args.overwrite,
    )?;
    runner.run();
    Ok(())
}

struct Runner<'a> {
    filter: BloomFilter,
    sh: xshell::Shell,
    input_files: Vec<PathBuf>,
    destination: &'a Path,
    dry_run: bool,
    overwrite: bool,
}

impl<'a> Runner<'a> {
    fn new(
        sh: Shell,
        source: &Path,
        destination: &'a Path,
        dry_run: bool,
        overwrite: bool,
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
        })
    }
    fn run(&mut self) -> anyhow::Result<()> {
        let mut duplicates: HashMap<PathBuf, HashSet<&Path>> = HashMap::new();

        for file_name in self.input_files.iter() {
            let metadata = Metadata::exiftool(&self.sh, &file_name)?;
            let new_file_name = metadata.new_file_name(self.destination);
            if self.filter.contains(&new_file_name) {
                Self::register_duplicate(&mut duplicates, &file_name, new_file_name);
            } else {
                self.move_file(&self.sh, &new_file_name)?;
                self.filter.insert(&new_file_name);
            }
        }
        self.move_duplicates(&self.sh, duplicates)
    }

    fn move_file(&self, sh: &Shell, new_file: &PathBuf) -> anyhow::Result<()> {
        todo!()
    }

    fn move_duplicates(
        &self,
        sh: &Shell,
        duplicates: HashMap<PathBuf, HashSet<&Path>>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn register_duplicate<'b>(
        duplicates: &mut HashMap<PathBuf, HashSet<&'b Path>>,
        file_name: &'b Path,
        new_file_name: PathBuf,
    ) {
        duplicates
            .entry(new_file_name)
            .or_default()
            .insert(file_name);
    }
}
