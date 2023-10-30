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
    #[arg(value_name = "sheet")]
    pub sheet_id: String,
}

#[derive(Parser, Clone, Debug, Default)]
pub struct FetchCmd {
    /// Sheet ID
    #[arg(value_name = "sheet")]
    pub sheet_id: Option<String>,

    /// Path to the Moodle CSV
    #[arg(long, value_name = "/path/to/csv")]
    pub csv: PathBuf,

    /// Start of the line range in the CSV
    #[arg(long, value_name = "line_number")]
    pub from_line: u64,

    /// End (inclusive!) of the line range in the CSV
    #[arg(long, value_name = "line_number")]
    pub to_line: u64,
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

/// Push Command Struct. Basically tells us whether we're dry-running.
#[derive(Parser, Clone, Debug, Default)]
pub struct PushCmd {
    #[arg(long = "dry-run", default_value_t = false)]
    pub dry_run: bool,
}

/// First subcommand ("verb") found on the cmdline
#[derive(Subcommand, Clone, Debug)]
pub enum Verb {
    /// Unpack/filter moodle zip using a csv file
    Unpack(UnpackFiles),
    /// Repack the zip to publish feedback/grades
    Repack(RepackDir),
    /// Initialize the master config file
    Init(MasterCfg),
    /// Grade team
    Grade(GradeCmd),
    /// Set up autofetch
    SetupFetch,
    /// Fetch assignment submissions
    Fetch(FetchCmd),
    /// Push grades to moodle
    Push(PushCmd),
}

#[derive(Parser, Clone, Debug)]
#[command(author, version, about, next_line_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub verb: Verb,
}
