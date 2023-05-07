mod args;
mod config;
mod grade;
mod gradingtable;
mod init;
mod repack;
mod unpack;

use clap::Parser;
use config::Structure;
use grade::grade;
use unpack::unpack;

use args::Verb;
use config::Grades;
use config::MasterCfg;
use config::UNPACK_GRADES_FILENAME;
use log::error;

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

    let master = match MasterCfg::resolve() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("MasterCfg::resolve() returned error {:?}", e);
            error!("could not read config. run \"kasm init\" first!");
            panic!()
        }
    };

    if master.unpack_structure != Structure::Groups {
        panic!("Only unpack_structure = \"Groups\" is currently supported.");
    }

    if master.repack_structure != Structure::Individuals {
        panic!("Only repack_structure = \"Individuals\" is currently supported.")
    }

    if master.recursive_unzip {
        error!("recursive unzip is not implemented yet, continuing");
    }

    let grades = Grades::resolve();

    match command.verb {
        Verb::Unpack(cfg) => {
            unpack(&master, &cfg).unwrap();
        }
        Verb::Grade(cfg) => {
            if let Ok(grades) = grades {
                grade(&master, &cfg, &grades).unwrap()
            } else {
                error!("{} could not be found!", UNPACK_GRADES_FILENAME);
            }
        }
        _ => todo!(),
    }
}
