use std::error::Error;

use crate::args::GradeCmd;
use crate::config::{Grades, MasterCfg};
use log::{error, info};

pub fn grade(master: &MasterCfg, cfg: &GradeCmd, grades: &Grades) -> Result<(), Box<dyn Error>> {
    let reg = regex::Regex::new(&master.groups_regex)?;

    let target = match &cfg.target {
        Some(str) => str.clone(),
        None => {
            let cd = std::env::current_dir()?;
            let infer = cd.components().rev().find_map(|c| {
                reg.captures(c.as_os_str().to_str().unwrap())
                    .and_then(|caps| caps.get(1).map(|c| c.as_str()))
            });

            if let Some(infer) = infer {
                info!("inferred group {} based on path", infer);
                infer.to_string()
            } else {
                error!("you didn't specify the group to be graded and it couldn't be inferred");
                return Err("".into());
            }
        }
    };

    info!("grading {} with {}", target, cfg.grade);

    let mut grades = grades.clone();
    let mut changed = false;
    grades
        .map
        .iter_mut()
        .find(|gd| {
            reg.captures(&gd.target).map_or(false, |caps| {
                caps.get(1).map_or(false, |cap| cap.as_str() == target)
            })
        })
        .map(|gd| {
            info!("found match");
            changed = true;
            gd.grade = cfg.grade.to_owned();
            Some(())
        })
        .or_else(|| {
            error!("no matching group found!");
            None
        });

    if changed {
        info!("writing grades");
        std::fs::write(grades.location.clone(), toml::to_string_pretty(&grades)?)?;
    }
    Ok(())
}
