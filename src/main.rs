mod args;
mod config;
mod init;
use clap::Parser;

use args::Verb;
use config::MasterCfg;

const DEF_LOG_LEVEL: &str = "debug";
const ENV_LOG_LEVEL: &str = "RUST_LOG";

fn main() {
    if std::env::var(ENV_LOG_LEVEL).is_err() {
        std::env::set_var(ENV_LOG_LEVEL, DEF_LOG_LEVEL);
    }
    pretty_env_logger::init();
    let command: args::Cli = args::Cli::parse();

    if let Verb::Init(ref cfg) = command.verb {
        init::init_master(cfg).unwrap();
        return;
    }

    let _master = MasterCfg::resolve().expect("master config");
}
