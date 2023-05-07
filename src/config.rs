use std::{error::Error, path::PathBuf};

use clap::*;
use log::debug;
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
    #[serde(skip)]
    #[clap(skip)]
    pub location: PathBuf,

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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Grades {
    #[serde(skip)]
    location: PathBuf,
    map: Vec<Grade>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Grade {
    r#for: String,
    grade: String,
}

fn find_in_preceding_dir_tree(filename: &str) -> Result<PathBuf, Box<dyn Error>> {
    let mut path = std::env::current_dir()?;

    while !path.join(filename).is_file() {
        if let Some(parent) = path.parent() {
            path = parent.to_path_buf();
        } else {
            return Err("couldn't find file".into());
        }
    }

    debug!("found {:?} under {:?}", filename, path);
    Ok(path.join(filename))
}

impl MasterCfg {
    pub fn resolve() -> Result<MasterCfg, Box<dyn Error>> {
        let cfg_path = find_in_preceding_dir_tree(MASTER_CFG_FILENAME)?;
        let mut cfg = toml::from_str::<MasterCfg>(&std::fs::read_to_string(cfg_path.clone())?)?;
        cfg.location = cfg_path;
        Ok(cfg)
    }
}

impl Grades {
    pub fn resolve() -> Result<Grades, Box<dyn Error>> {
        let cfg_path = find_in_preceding_dir_tree(UNPACK_GRADES_FILENAME)?;
        let mut cfg = toml::from_str::<Grades>(&std::fs::read_to_string(cfg_path.clone())?)?;
        cfg.location = cfg_path;
        Ok(cfg)
    }
}
