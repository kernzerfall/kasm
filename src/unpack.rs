use std::error::Error;
use std::path::Path;
use std::path::PathBuf;

use crate::args::UnpackFiles;
use crate::config::Grades;
use crate::config::MasterCfg;

use crate::gradingtable::GradingRecord;
use log::{error, info, trace};
use regex::Regex;

use crate::config::Grade;
use crate::config::Structure;
use crate::config::UNPACK_CSV_FILENAME;
use crate::config::UNPACK_GRADES_FILENAME;
use crate::config::UNPACK_PATH_FILENAME_BASE;

pub fn unpack(master: &MasterCfg, cfg: &UnpackFiles) -> Result<(), Box<dyn Error>> {
    let unpack_path: PathBuf = (UNPACK_PATH_FILENAME_BASE.to_owned() + &cfg.sheet_id).into();
    if unpack_path.is_dir() {
        error!("unpack path {:?} already exists!", unpack_path);
        return Err("".into());
    }

    info!("creating dir {:?}", unpack_path);
    std::fs::create_dir_all(unpack_path.clone())?;

    let reg = regex::Regex::new(&master.groups_regex)?;
    let records = GradingRecord::from_csv(&cfg.moodle_csv.clone()).expect("gradingtable csv");

    info!("csv has {} records", records.len());
    let filtered = records
        .iter()
        .filter(|&r| {
            reg.captures(&r.group).map_or(false, |caps| {
                caps.get(1)
                    .map_or(false, |val| val.as_str() == master.group)
            })
        })
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        error!(
            "could not find any records matching master.group = {}",
            master.group
        );
        return Err("".into());
    }

    info!(
        "found {} records matching master.group = {}",
        filtered.len(),
        master.group
    );

    gen_grading_files(master, cfg, &unpack_path, filtered)?;
    unzip_filter_main(master, cfg, &reg, &unpack_path)?;

    Ok(())
}

fn gen_grading_files(
    master: &MasterCfg,
    cfg: &UnpackFiles,
    unpack_path: &Path,
    filtered: Vec<&GradingRecord>,
) -> Result<(), Box<dyn Error>> {
    let nested_csv_path = unpack_path.join(UNPACK_CSV_FILENAME);
    let mut grades_arr: Vec<Grade> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    let nested_csv = std::fs::File::create(nested_csv_path)?;

    info!("writing filtered csv");
    let mut wtr = csv::Writer::from_writer(nested_csv);
    filtered.iter().for_each(|&r| {
        trace!("serializing {:?}", r);
        wtr.serialize(r).unwrap();

        if !seen.contains(&r.group) {
            seen.push(r.group.clone());

            if master.unpack_structure == Structure::Groups {
                grades_arr.push(Grade {
                    target: r.group.to_owned(),
                    grade: r.best_grade.to_owned(),
                })
            }
        }

        if master.unpack_structure == Structure::Individuals {
            grades_arr.push(Grade {
                target: r.uni_id.to_owned(),
                grade: r.best_grade.to_owned(),
            });
        }
    });
    wtr.flush()?;

    info!("saw {} discreet groups", seen.len());

    let grades_toml_path = unpack_path.join(UNPACK_GRADES_FILENAME);
    info!("writing grades.toml");
    std::fs::write(
        grades_toml_path.clone(),
        toml::to_string_pretty(&Grades {
            location: grades_toml_path,
            map: grades_arr,
            sheet_id: cfg.sheet_id.to_owned(),
        })?,
    )?;

    Ok(())
}

fn unzip_filter_main(
    master: &MasterCfg,
    cfg: &UnpackFiles,
    reg: &Regex,
    unpack_path: &Path,
) -> Result<(), Box<dyn Error>> {
    info!("unzipping main zip file");

    let file = std::fs::File::open(&cfg.moodle_zip)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut curr = archive.by_index(i)?;
        let curr_name = curr.name();

        if reg.captures(curr_name).map_or(false, |caps| {
            caps.get(1)
                .map_or(false, |cap| cap.as_str() == master.group)
        }) {
            let enclosed_path = curr.enclosed_name().unwrap();
            let subdir = enclosed_path
                .components()
                .next()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
                .split_once('_')
                .unwrap()
                .0;

            let extr = unpack_path
                .join(subdir)
                .join(enclosed_path.file_name().unwrap());
            std::fs::create_dir_all(extr.parent().unwrap())?;
            let mut target = std::fs::File::create(extr)?;

            std::io::copy(&mut curr, &mut target)?;
        };
    }

    Ok(())
}
