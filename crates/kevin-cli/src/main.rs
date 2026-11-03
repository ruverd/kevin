mod args;
mod cli;
mod evaluation;
mod provider;
mod skills;
mod storage;
mod wizard;

use clap::Parser;
fn main() {
    if let Err(error) = cli::run(args::Args::parse()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
