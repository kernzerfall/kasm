use std::path::PathBuf;

use clap::*;
use serde::{Deserialize, Serialize};
use strum::Display;

#[allow(dead_code)]
pub const MASTER_CFG_FILENAME: &str = "kasm.toml";
pub const DEFAULT_GROUPS_REGEX: &str = r#"\([0-9]{2}\).+\([0-9]{2}\)"#;
#[allow(dead_code)]
pub const UNPACK_PATH_FILENAME_HB: &str = "unpack_{{sheet-id}}";
#[allow(dead_code)]
pub const UNPACK_CSV_FILENAME: &str = "original.csv";
#[allow(dead_code)]
pub const UNPACK_GRADES_FILENAME: &str = "grades.toml";
#[allow(dead_code)]
pub const UNPACK_SLAVE_CFG_FILENAME: &str = "kasm.slave.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SlaveConfig {
    sheet_id: String,
    csv_path: PathBuf,
    unpack_path: PathBuf,
}

#[derive(ValueEnum, Clone, Debug, Default, Display, Serialize, Deserialize)]
pub enum Structure {
    #[default]
    Groups,
    Individuals,
}

#[derive(Parser, Clone, Debug, Default, Serialize, Deserialize)]
pub struct MasterCfg {
    /// Regex with for (group, team)
    #[arg(short = 'r', long = "regex", value_name = "expr", default_value = DEFAULT_GROUPS_REGEX)]
    pub groups_regex: String,

    // Your exercise group number as shown in Moodle
    #[arg(short = 'g', long = "group", value_name = "group id")]
    pub group: String,

    /// Unzip nested zip files
    #[arg(long, default_value_t = false)]
    pub recursive_unzip: bool,

    /// Regex for files to repack
    #[arg(short = 'f', long, value_name = "expr")]
    pub repack_filter: Option<String>,

    #[arg(long, value_name = "struct", default_value = "groups")]
    pub unpack_structure: Structure,

    #[arg(long, value_name = "struct", default_value = "groups")]
    pub repack_structure: Structure,
}
