mod args;
mod config;
use clap::Parser;

fn main() {
    let _command: args::Cli = args::Cli::parse();
}
