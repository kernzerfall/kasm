mod args;
mod config;
mod init;
mod unpack;
use clap::Parser;
use config::Structure;
use unpack::unpack;
mod gradingtable;

use args::Verb;
use config::Grades;
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

    let master = MasterCfg::resolve().expect("master config");

    if master.unpack_structure != Structure::Groups {
        panic!("Only unpack_structure = \"Groups\" is currently supported.");
    }

    let _grades = Grades::resolve();

    match command.verb {
        Verb::Unpack(cfg) => {
            unpack(&master, &cfg).unwrap();
        }
        _ => todo!(),
    }
}
