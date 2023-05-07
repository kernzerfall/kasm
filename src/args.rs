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

/// Grade Command struct. Identical to config::Grade, but
/// kept separate due to semantical differences between
/// the target variables.
#[derive(Parser, Clone, Debug, Default)]
pub struct GradeCmd {
    /// The grade to assign the group, as Moodle wants it
    ///
    /// e.g. 10,5 or 10,0
    #[arg(value_name = "grade")]
    pub grade: String,

    /// ID of the group/person to grade.
    ///
    /// Inferred if ommitted.
    /// Matched by the 2nd regex capture group!  
    ///
    /// e.g. 04 or K or 01
    // note that this MUST come second if we want to omit/infer it...
    #[arg(value_name = "target_team")]
    pub target: Option<String>,
}

/// First subcommand ("verb") found on the cmdline
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
    /// Grade team
    Grade(GradeCmd),
}

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, next_line_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub verb: Verb,
}
