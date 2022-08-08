mod cli;
mod fs;

use clap::Parser as _;

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    for f in fs::scan(&args.source)? {
        println!("{}", f.display())
    }
    Ok(())
}
