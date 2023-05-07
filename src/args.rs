use crate::config::MasterCfg;
use clap::*;
use std::path::PathBuf;

#[derive(Parser, Clone, Debug, Default)]
pub struct UnpackFiles {
    #[arg(short, long = "sheet", value_name = "sheet")]
    pub sheet_id: String,

    /// Path to the .zip downloaded from moodle
    #[arg(short = 'z', long, value_name = "/path/to/zip")]
    pub moodle_zip: PathBuf,

    /// Path to the .csv downloaded from moodle
    #[arg(short = 'c', long, value_name = "/path/to/csv")]
    pub moodle_csv: PathBuf,
}

#[derive(Parser, Clone, Debug, Default)]
pub struct RepackDir {
    /// Sheet ID
    #[arg(short, long = "sheet", value_name = "sheet")]
    pub sheet_id: String,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Verb {
    /// Unpack/filter moodle zip using a csv file
    Unpack(UnpackFiles),
    /// Repack the zip to publish feedback/grades
    Repack,
    /// Initialize the master config file
    Init(MasterCfg),
    /// Copy config from other location
    CopyConfig,
}

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, next_line_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub verb: Verb,
}
