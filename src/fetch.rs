use std::{collections::HashMap, error::Error, time::Duration};

use crate::config::MasterCfg;
use log::{error, info, warn};

const KEYRING_SERVICE_NAME: &str = "kasm-moodle-token";
static MOODLE_REST_URL: &str = "https://moodle.rwth-aachen.de/webservice/rest/server.php";

pub fn setup(master: &MasterCfg) -> core::result::Result<(), Box<dyn Error>> {
    let overwrite_course;

    let mut new_master = master.clone();
    let user = whoami::username();

    if let Some(course) = &master.moodle_course_id {
        info!("the saved course id is {}.", course);
        print!("overwrite? (y/N) > ");
        overwrite_course = inquire::Confirm::new("overwrite? (y/n) > ")
            .prompt_skippable()?
            .unwrap_or(false);
    } else {
        overwrite_course = true;
    }

    if overwrite_course {
        new_master.moodle_course_id = inquire::Text::new("Course ID > ").prompt()?.into();
    }

    info!(
        "using course id = {{{}}}",
        new_master.moodle_course_id.as_ref().unwrap()
    );

    let entry = keyring::Entry::new(KEYRING_SERVICE_NAME, &user)?;
    let overwrite_token = match entry.get_password() {
        Err(keyring::Error::NoEntry) => true,
        Err(keyring::Error::Ambiguous(a)) => {
            error!("{} is ambiguous in your keyring", KEYRING_SERVICE_NAME);
            return Err(keyring::Error::Ambiguous(a).into());
        }
        Err(e) => return Err(e.into()),
        Ok(_) => {
            info!(
                "your keyring already contains an entry for {}",
                KEYRING_SERVICE_NAME
            );
            inquire::Confirm::new("overwrite? (y/n) > ")
                .prompt_skippable()?
                .unwrap_or(false)
        }
    };

    if overwrite_token {
        let token = inquire::Password::new("Moodle Token (won't be echoed): ")
            .without_confirmation()
            .prompt()?;
        info!(
            "saving token to keyring ({}, {})",
            KEYRING_SERVICE_NAME, user
        );
        entry.set_password(&token)?;
    }

    std::fs::write(
        new_master.location.clone(),
        toml::to_string_pretty(&new_master)?,
    )?;
    Ok(())
}

pub struct MoodleFetcher {
    pub course_id: String,
    token: String,
}

impl MoodleFetcher {
    pub fn new(course_id: String) -> MoodleFetcher {
        let entry =
            keyring::Entry::new(KEYRING_SERVICE_NAME, &whoami::username()).unwrap_or_else(|e| {
                error!("{}", e);
                error!("run fetch-setup first!");
                panic!();
            });

        MoodleFetcher {
            course_id,
            token: entry.get_password().unwrap(),
        }
    }

    pub fn fetch_directory(&self) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let resp = reqwest::blocking::Client::new()
            .get(MOODLE_REST_URL)
            .query(&[
                ("moodlewsrestformat", "json"),
                ("wsfunction", "mod_assign_get_assignments"),
                ("wstoken", &self.token),
                ("courseids[0]", &self.course_id),
            ])
            .send()?;

        let data: serde_json::Value = serde_json::de::from_str(&resp.text()?)?;
        Ok(data
            .get("courses")
            .unwrap()
            .get(0)
            .unwrap()
            .get("assignments")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|assignment| {
                (
                    assignment.get("name").unwrap().to_string(),
                    assignment.get("id").unwrap().to_string(),
                )
            })
            .collect::<HashMap<String, String>>())
    }

    pub fn get_submissions_list(&self, assignment_id: &String) -> Result<(), Box<dyn Error>> {
        info!("getting submission list");
        warn!("this is going to take an eternity in big course pages");
        warn!("compare how slow moodle is when you click on 'view all submissions'");
        warn!("go brew a coffee or touch grass or something");
        let resp = reqwest::blocking::Client::new()
            .get(MOODLE_REST_URL)
            .query(&[
                ("moodlewsrestformat", "json"),
                ("wsfunction", "mod_assign_get_submissions"),
                ("wstoken", &self.token),
                ("assignmentids[0]", &assignment_id),
            ])
            .timeout(Duration::new(900, 0)) // 15 Min Timeout because Moodle is _fast_
            .send()?;

        let data: serde_json::Value = serde_json::from_str(&resp.text()?)?;
        let submissions = data;

        println!("{:#?}", submissions);

        Ok(())
    }

    pub fn get_participants_info(&self, assignment_id: &String) {
        let resp = reqwest::blocking::Client::new()
            .get(MOODLE_REST_URL)
            .query(&[
                ("moodlewsrestformat", "json"),
                ("wsfunction", "mod_assign_list_participants"),
                ("wstoken", &self.token),
                ("assignid", assignment_id),
                ("groupid", "0"),
                ("onlyids", "1"),
                ("filter", ""),
            ]);
    }

    pub fn interactive_dl(&self) -> Result<(), Box<dyn Error>> {
        let assignments = self.fetch_directory()?;
        let selected = inquire::Select::new(
            "Select an assignment to download",
            assignments.keys().collect(),
        )
        .prompt()?;

        let dl_id = assignments.get(selected).unwrap();

        self.get_submissions_list(dl_id)?;

        Ok(())
    }
}
