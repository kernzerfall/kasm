use std::{error::Error, path::PathBuf};

use clap::*;
use log::debug;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::gradingtable::GradingRecord;

pub const MASTER_CFG_FILENAME: &str = "kasm.toml";
pub const DEFAULT_GROUPS_REGEX: &str = r#"([0-9]{1,3})"#;
pub const UNPACK_PATH_FILENAME_BASE: &str = "unpack_";
pub const UNPACK_CSV_FILENAME: &str = ".filtered.csv";
pub const UNPACK_GRADES_FILENAME: &str = "grades.toml";

/// Tells us whether the zip we're extracting contains groupped or individual
/// submissions, as well as whether we want to repack it as one or the other.
#[derive(ValueEnum, Clone, Debug, Default, Display, Serialize, Deserialize, PartialEq, Eq)]
pub enum Structure {
    #[default]
    Groups,
    Individuals,
}

/// Definition of the master config file
/// (default: kasm.toml)
#[derive(Parser, Clone, Debug, Default, Serialize, Deserialize)]
pub struct MasterCfg {
    #[serde(skip)]
    #[clap(skip)]
    pub location: PathBuf,

    #[serde(skip)]
    #[clap(skip)]
    pub moodle_token: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long = "course-id", value_name = "12345678")]
    pub moodle_course_id: Option<String>,

    /// Regex with for (group, team)
    #[arg(short = 'r', long = "regex", value_name = "expr", default_value = DEFAULT_GROUPS_REGEX)]
    pub groups_regex: String,

    // Your exercise group number as shown in Moodle
    #[arg(short = 'g', long = "group", value_name = "group id")]
    pub group: Option<String>,

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

/// Where the files came from
#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum Source {
    #[default]
    CsvAndZip,
    Autofetch,
}

/// Nested grades.toml definition
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Grades {
    /// We save internally where we found the file so
    /// that we don't need to search for it again when
    /// we want to overwrite it later.
    #[serde(skip)]
    pub location: PathBuf,

    /// Moodle Assignment ID. Used in autofetch workflows to push grades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assign_id: Option<String>,

    /// Indicates the source of the files.
    /// Currently it's impossible to mix and match autofetch
    /// and .zips .csvs.
    pub source: Source,

    /// Sheed Identificator
    /// e.g. 04
    pub sheet_id: String,

    /// Grade maps
    /// target (matrnr/group_id) -> grade
    pub map: Vec<Grade>,
}

impl Grades {
    // Generates a vector of grading records from the filtered csv
    // for the given group of students by overwriting grades using
    // the current object's grades
    pub fn collect_students_for_group(
        &self,
        gt: &[GradingRecord],
        group_id: &str,
    ) -> Vec<GradingRecord> {
        gt.to_owned()
            .iter()
            .filter(|&gr| gr.group == group_id)
            .filter_map(|gr| {
                let mut new_gr = gr.clone();
                new_gr.grade = self.find_grade_for_target(group_id)?;
                Some(new_gr)
            })
            .collect()
    }

    pub fn find_grade_for_target(&self, target: &str) -> Option<String> {
        self.map
            .iter()
            .find(|&g| g.target == target)
            .map(|g| g.grade.clone())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Grade {
    /// Full team name/matrnr that the grade corresponds to
    pub target: String,

    /// Internal Moodle ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internal_id: Option<String>,

    /// Member `userid`s to push grades to. Applies only to auto-workflow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<String>>,

    /// The grade to assign the group, should be formatted
    /// as Moodle wants it to be formatted
    /// e.g. 10,5 or 10,0
    pub grade: String,
}

/// Walks upwards the directory tree and tries to find `filename`
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
    /// Finds/parses the master config
    pub fn resolve() -> Result<MasterCfg, Box<dyn Error>> {
        let cfg_path = find_in_preceding_dir_tree(MASTER_CFG_FILENAME)?;
        let mut cfg = toml::from_str::<MasterCfg>(&std::fs::read_to_string(cfg_path.clone())?)?;
        cfg.location = cfg_path;
        Ok(cfg)
    }
}

impl Grades {
    /// Finds/parses the nested grades config
    pub fn resolve() -> Result<Grades, Box<dyn Error>> {
        let cfg_path = find_in_preceding_dir_tree(UNPACK_GRADES_FILENAME)?;
        let mut cfg = toml::from_str::<Grades>(&std::fs::read_to_string(cfg_path.clone())?)?;
        cfg.location = cfg_path;
        Ok(cfg)
    }
}
