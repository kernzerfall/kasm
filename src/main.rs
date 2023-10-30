use clap::Parser;
use kasm::config::Structure;
use kasm::config::UNPACK_PATH_FILENAME_BASE;
use kasm::grade::grade;
use kasm::repack::repack;
use kasm::unpack::unpack;

use kasm::args::Verb;
use kasm::config::Grades;
use kasm::config::MasterCfg;
use kasm::config::UNPACK_GRADES_FILENAME;
use log::{error, info};

const DEF_LOG_LEVEL: &str = "info";
const ENV_LOG_LEVEL: &str = "RUST_LOG";

fn main() {
    if std::env::var(ENV_LOG_LEVEL).is_err() {
        std::env::set_var(ENV_LOG_LEVEL, DEF_LOG_LEVEL);
    }
    pretty_env_logger::init();
    let command = kasm::args::Cli::parse();

    if let Verb::Init(ref cfg) = command.verb {
        kasm::init::init_master(cfg).unwrap();
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
        Verb::Repack(cfg) => {
            repack(&master, &cfg).unwrap();
        }
        Verb::SetupFetch => {
            kasm::fetch::setup(&master).unwrap();
        }
        Verb::Fetch(cfg) => {
            kasm::fetch::MoodleFetcher::new(&master)
                .interactive_dl(cfg)
                .unwrap();
        }
        Verb::Push(cfg) => {
            if let Ok(ref grades) = grades {
                kasm::fetch::MoodleFetcher::new(&master)
                    .push_grades(grades, cfg.dry_run)
                    .unwrap();
            } else {
                error!("{} could not be found!", UNPACK_GRADES_FILENAME);
                info!(
                    "run this command from inside an {}xx directory",
                    UNPACK_PATH_FILENAME_BASE
                );
            }
        }
        _ => panic!("unexpected verb"),
    }
}
