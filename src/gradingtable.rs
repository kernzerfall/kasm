use log::trace;
use std::{error::Error, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradingRecord {
    /// Internal moodle id
    #[serde(rename = "ID")]
    pub internal_id: String,

    /// Full name
    #[serde(rename = "Vollständiger Name")]
    pub name: String,

    /// University ID Number (MatrNr.)
    #[serde(rename = "Matrikelnummer")]
    pub uni_id: String,

    #[serde(rename = "Status")]
    pub status: String,

    /// Full group name
    #[serde(rename = "Gruppe")]
    pub group: String,

    // Other fields
    #[serde(rename = "Bewertung")]
    pub grade: String,

    #[serde(rename = "Bestwertung")]
    pub best_grade: String,

    #[serde(rename = "Bewertung kann geändert werden")]
    pub grade_locked: String,

    #[serde(rename = "Zuletzt geändert (Abgabe)")]
    pub last_change_submission: String,

    #[serde(rename = "Zuletzt geändert (Bewertung)")]
    pub last_change_grade: String,

    #[serde(rename = "Feedback als Kommentar")]
    pub feedback_comment: String,
}

impl GradingRecord {
    pub fn from_csv(path: &PathBuf) -> Result<Vec<GradingRecord>, Box<dyn Error>> {
        if !path.is_file() {
            return Err("could not find csv".into());
        }

        let file = std::fs::File::open(path)?;
        let mut reader = csv::Reader::from_reader(file);
        Ok(reader
            .deserialize()
            .filter_map(|result| {
                trace!("checking {:?}", result);
                result.ok()
            })
            .collect::<Vec<GradingRecord>>())
    }
}
