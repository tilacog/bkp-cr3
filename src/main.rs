mod cli;
mod exif;
mod fs;

use clap::Parser as _;

use crate::exif::Metadata;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let sh = xshell::Shell::new()?;
    for f in fs::scan(&args.source)? {
        let metadata = Metadata::exiftool(&sh, &f)?;
        println!(
            "{}\t->\t{}",
            f.display(),
            metadata.new_file_name(&args.destination).display()
        );
    }
    Ok(())
}
