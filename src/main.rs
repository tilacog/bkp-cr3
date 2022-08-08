mod cli;

use clap::Parser as _;

fn main() {
    let args = cli::Cli::parse();
    dbg!(args);
}
