use serde_json::Value;
use std::{collections::HashMap, error::Error, path::PathBuf, time::Duration};

use crate::config::{Grade, Grades, MasterCfg, UNPACK_GRADES_FILENAME, UNPACK_PATH_FILENAME_BASE};
use log::{error, info, warn};

const KEYRING_SERVICE_NAME: &str = "kasm-moodle-token";
static MOODLE_REST_URL: &str = "https://moodle.rwth-aachen.de/webservice/rest/server.php";

pub fn setup(master: &MasterCfg) -> core::result::Result<(), Box<dyn Error>> {
    let overwrite_course;

    let mut new_master = master.clone();
    let user = whoami::username();

    if let Some(course) = &master.moodle_course_id {
        info!("the saved course id is {}.", course);
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

#[derive(Debug, Clone)]
pub struct SubmissionFileMap {
    pub dl_path: PathBuf,
    pub dl_url: String,
    pub group_id: String,
    pub group_name: String,
}

pub struct MoodleFetcher {
    pub course_id: String,
    pub config: MasterCfg,
    token: String,
}

impl MoodleFetcher {
    pub fn new(config: &MasterCfg) -> MoodleFetcher {
        let entry =
            keyring::Entry::new(KEYRING_SERVICE_NAME, &whoami::username()).unwrap_or_else(|e| {
                error!("{}", e);
                error!("run setup-fetch first!");
                panic!();
            });

        MoodleFetcher {
            config: config.clone(),
            course_id: config
                .moodle_course_id
                .as_ref()
                .unwrap_or_else(|| {
                    error!("run fetch-setup first!");
                    panic!()
                })
                .clone(),
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
                    assignment
                        .get("name")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string(),
                    assignment.get("id").unwrap().to_string(),
                )
            })
            .collect::<HashMap<String, String>>())
    }

    pub fn get_submissions_list(
        &self,
        assignment_id: &String,
    ) -> Result<HashMap<String, Vec<Value>>, Box<dyn Error>> {
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
                ("assignmentids[0]", assignment_id),
            ])
            .timeout(Duration::new(900, 0)) // 15 Min Timeout because Moodle is _fast_
            .send()?;

        let rt = &resp.text()?;
        let parsed: Value = serde_json::from_str(rt)?;

        let gid_plug_arrs: HashMap<String, Vec<Value>> = parsed
            .get("assignments")
            .unwrap()
            .get(0)
            .unwrap()
            .get("submissions")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|submission| {
                (submission.get("groupid")?, submission.get("plugins")?).into()
            })
            .map(|(a, b)| {
                (
                    a.to_string(),
                    b.as_array()
                        .unwrap()
                        .iter()
                        .filter_map(|plug| plug.get("fileareas")?.as_array())
                        .flatten()
                        .filter(|fa| fa.get("area").unwrap() == "submission_files")
                        .filter_map(|fa| fa.get("files"))
                        .filter_map(|ff| ff.as_array())
                        .flatten()
                        .cloned()
                        .collect(),
                )
            })
            .collect();

        Ok(gid_plug_arrs)
    }

    pub fn get_group_mappings(
        &self,
        assignment_id: &String,
    ) -> Result<(HashMap<String, String>, HashMap<String, Vec<String>>), Box<dyn Error>> {
        info!("fetching participants list");
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
            ])
            .timeout(Duration::new(900, 0))
            .send()?;
        let rt = resp.text()?;

        let parsed: Value = serde_json::from_str(&rt)?;
        let mut group_members_mappings: HashMap<String, Vec<String>> = HashMap::new();

        let mut groups: HashMap<String, String> = HashMap::new();
        parsed
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|part| {
                (
                    part.get("id")?,
                    part.get("groupname")?,
                    part.get("groupid")?,
                    part.get("submissionstatus")?,
                )
                    .into()
            })
            .filter(|(_, _, _, status)| status.as_str().unwrap() == "submitted")
            .for_each(|(userid, gname, gid, _)| {
                group_members_mappings
                    .entry(gid.to_string())
                    .or_insert_with(Vec::new)
                    .push(userid.to_string());
                groups
                    .entry(gid.to_string())
                    .or_insert_with(|| gname.as_str().unwrap().into());
            });

        Ok((groups, group_members_mappings))
    }

    pub fn interactive_dl(&mut self) -> Result<(), Box<dyn Error>> {
        let assignments = self.fetch_directory()?;
        let mut prompt_revord: Vec<&String> = assignments.keys().collect();
        prompt_revord.sort_unstable();
        prompt_revord.reverse();
        let selected =
            inquire::Select::new("Select an assignment to download", prompt_revord).prompt()?;

        let reg = regex::Regex::new(&self.config.groups_regex)?;
        let dl_id = assignments.get(selected).unwrap();

        let participants = self.get_group_mappings(dl_id)?;
        let submissions = self.get_submissions_list(dl_id)?;

        let nr_regex = regex::Regex::new("([0-9]{1,2})")?;

        let base_path = PathBuf::from(
            UNPACK_PATH_FILENAME_BASE.to_string()
                + nr_regex
                    .captures(selected.as_str())
                    .unwrap()
                    .get(1)
                    .unwrap()
                    .into(),
        );
        info!("Sel {:?}", base_path);

        let filtered_participants: HashMap<&String, &String> = participants
            .0
            .iter()
            .filter_map(|(k, v)| {
                if reg.captures(v)?.get(1)?.as_str() == self.config.group {
                    Some((k, v))
                } else {
                    None
                }
            })
            .collect();

        let sheet_id = selected.split(' ').last().unwrap().to_string();
        let loc = format!("{}{}", UNPACK_PATH_FILENAME_BASE, sheet_id);
        let mut config = Grades {
            location: loc.into(),
            sheet_id: selected.split(' ').last().unwrap().to_string(),
            map: Default::default(),
            source: crate::config::Source::Autofetch,
            assign_id: Some(dl_id.to_owned()),
        };
        self.gen_grading_files(&mut config, &filtered_participants, &participants.1)?;

        let filtered_files: Vec<SubmissionFileMap> = filtered_participants
            .iter()
            .filter(|(_, v)| reg.captures(v).unwrap().get(1).unwrap().as_str() == self.config.group)
            .flat_map(|(gid, gname)| {
                let group_path = base_path.join(gname);
                submissions
                    .get(gid.as_str())
                    .unwrap()
                    .iter()
                    .filter_map(move |file| {
                        SubmissionFileMap {
                            dl_url: file.get("fileurl")?.as_str()?.to_string(),
                            dl_path: group_path.join(file.get("filename")?.as_str()?),
                            group_id: gid.to_string(),
                            group_name: gname.to_string(),
                        }
                        .into()
                    })
            })
            .collect();

        info!("downloading {} file(s)", filtered_files.len());
        for file in &filtered_files {
            info!("downloading submission of {{{}}}", file.group_name);
            std::fs::create_dir_all(file.dl_path.parent().unwrap())?;
            let resp = reqwest::blocking::Client::new()
                .get(&file.dl_url)
                .query(&[("token", self.token.as_str())])
                .timeout(Duration::new(900, 0))
                .send()?;

            std::fs::write(&file.dl_path, resp.bytes()?)?;
        }

        info!("done");
        warn!("note: repacking autofetched files is not implemented yet!");

        Ok(())
    }
    fn gen_grading_files(
        &self,
        conf: &mut Grades,
        groups: &HashMap<&String, &String>,
        group_user_mappings: &HashMap<String, Vec<String>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut grades_arr: Vec<Grade> = Vec::new();
        let mut seen: Vec<String> = Vec::new();

        groups.iter().for_each(|(&gid, &gname)| {
            grades_arr.push(Grade {
                grade: "0".to_string(),
                internal_id: Some(gid.to_owned()),
                members: group_user_mappings.get(gid).cloned(),
                target: gname.to_owned(),
            });
            seen.push(gid.to_owned());
        });

        info!("saw {} discreet groups", seen.len());

        let grades_toml_path = conf.location.join(UNPACK_GRADES_FILENAME);

        info!("writing {:#?}", grades_toml_path);
        std::fs::create_dir_all(&conf.location)?;
        std::fs::write(
            grades_toml_path.clone(),
            toml::to_string_pretty(&Grades {
                location: grades_toml_path,
                map: grades_arr,
                sheet_id: conf.sheet_id.to_owned(),
                source: conf.source.to_owned(),
                assign_id: conf.assign_id.to_owned(),
            })?,
        )?;

        Ok(())
    }

    fn set_grade_for(
        &self,
        assignid: String,
        userid: String,
        grade: String,
        dry_run: bool,
    ) -> Result<(), Box<dyn Error>> {
        if dry_run {
            let userdata_req = reqwest::blocking::Client::new()
                .get(MOODLE_REST_URL)
                .query(&[
                    ("moodlewsrestformat", "json"),
                    ("wsfunction", "mod_assign_get_participant"),
                    ("wstoken", self.token.as_str()),
                    ("assignid", assignid.as_str()),
                    ("userid", userid.as_str()),
                ])
                .timeout(Duration::new(900, 0))
                .send()?;

            let userdata: serde_json::Value = serde_json::from_reader(userdata_req)?;
            let uname = userdata.get("fullname").unwrap().as_str().unwrap();
            let gname = userdata.get("groupname").unwrap().as_str().unwrap();

            info!("dry-run: would set {grade} for ({uname}) AND Group ({gname})");
            return Ok(());
        }

        let adj_grade = grade.replace(',', ".");

        info!("Grading {userid} with {grade}");
        let req = reqwest::blocking::Client::new()
            .get(MOODLE_REST_URL)
            .query(&[
                ("moodlewsrestformat", "json"),
                ("wsfunction", "mod_assign_save_grade"),
                ("wstoken", self.token.as_str()),
                // Moodle IDs
                ("assignmentid", assignid.as_str()),
                ("userid", userid.as_str()),
                // Grade latest attempt
                ("attemptnumber", "-1"),
                // Set to graded
                ("workflowstate", "graded"),
                // Do not allow another attempt
                ("addattempt", "0"),
                // The grade itself
                ("grade", adj_grade.as_str()),
                // Apply to whole group
                ("applytoall", "1"),
                // Text Feedback (not implemented yet but Moodle needs it to be
                // here)
                ("plugindata[assignfeedbackcomments_editor][text]", ""),
                ("plugindata[assignfeedbackcomments_editor][format]", "4"),
            ])
            .timeout(Duration::new(900, 0))
            .send()?;

        log::debug!("{:#?}", req.url());
        log::debug!("{}", req.text()?);

        Ok(())
    }

    pub fn push_grades(&mut self, grades: &Grades, dry_run: bool) -> Result<(), Box<dyn Error>> {
        let assign_id = grades
            .assign_id
            .clone()
            .expect("Moodle Assignment ID in grades.toml");

        grades.map.iter().for_each(|record| {
            let members = record.members.clone().unwrap();
            self.set_grade_for(
                assign_id.to_owned(),
                members.get(0).unwrap().to_owned(),
                record.grade.to_owned(),
                dry_run,
            )
            .expect("success setting grade");
        });

        Ok(())
    }
}
