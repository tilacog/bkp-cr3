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
    let sh = Shell::new()?;
    let input_files = fs::scan(&args.source)?;
    let mut processed_files: HashMap<PathBuf, HashSet<PathBuf>> = HashMap::new();

    // parse metadata and group
    for source in input_files.into_iter() {
        let metadata = Metadata::exiftool(&sh, &source)?;
        let dest = metadata.new_file_name(&args.destination);
        processed_files.entry(dest).or_default().insert(source);
    }

    // fan out
    for (destination, sources) in processed_files.into_iter() {
        if sources.len() == 1 {
            let source = sources.into_iter().next().unwrap();
            handle_file(&sh, &source, &destination, &args)?;
        } else {
            handle_duplicates(&sh, &destination, sources, &args)?
        }
    }
    Ok(())
}

fn handle_file(
    sh: &Shell,
    source: &Path,
    destination: &Path,
    args: &cli::Cli,
) -> anyhow::Result<()> {
    if args.dry_run {
        println!("{} -> {}", source.display(), destination.display());
        Ok(())
    } else {
        fs::handle_file(sh, source, destination, args.do_move, args.overwrite)
    }
}

fn handle_duplicates(
    sh: &Shell,
    destination_prefix: &Path,
    sources: HashSet<PathBuf>,
    args: &cli::Cli,
) -> anyhow::Result<()> {
    let mut counter: u32 = 1;
    for old in sources.into_iter() {
        let destination_with_suffix = fs::increment_name(destination_prefix, counter);
        handle_file(sh, &old, &destination_with_suffix, args)?;
        counter += 1
    }
    Ok(())
}
