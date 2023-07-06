use log::{debug, error, info};
use std::io::Write;
use std::{
    error::Error,
    fs::File,
    path::PathBuf,
    time::{self, UNIX_EPOCH},
};

use crate::config::Structure;
use crate::{
    args::RepackDir,
    config::{
        Grades, MasterCfg, UNPACK_CSV_FILENAME, UNPACK_GRADES_FILENAME, UNPACK_PATH_FILENAME_BASE,
    },
    gradingtable::GradingRecord,
};

pub fn repack(master: &MasterCfg, cfg: &RepackDir) -> Result<(), Box<dyn Error>> {
    let unpacked_path: PathBuf = (UNPACK_PATH_FILENAME_BASE.to_string() + &cfg.sheet_id).into();

    let packing_time = time::SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string();

    // Build the repacked zip name
    let zip_name: PathBuf = format!("feedback_{}_{}.zip", cfg.sheet_id, &packing_time).into();

    // Build the new csv name
    let grading_csv_name: PathBuf = format!("grades_{}_{}.csv", cfg.sheet_id, &packing_time).into();

    // Check that we have all that we need
    // - Unpacked dir
    // - grades.toml
    // - .filtered.csv

    if !unpacked_path.is_dir() {
        error!("the {:?} is not a directory", unpacked_path);
        return Err("".into());
    }

    if !unpacked_path.join(UNPACK_GRADES_FILENAME).is_file() {
        error!("grades file could not be found in {:?}", unpacked_path);
        return Err("".into());
    }

    if !unpacked_path.join(UNPACK_CSV_FILENAME).is_file() {
        error!(
            "filtered csv file could not be found in {:?}",
            unpacked_path
        );
        return Err("".into());
    }

    // Parse stuff
    let grading_table = GradingRecord::from_csv(&unpacked_path.join(UNPACK_CSV_FILENAME))?;
    let grades: Grades = toml::from_str(&std::fs::read_to_string(
        unpacked_path.join(UNPACK_GRADES_FILENAME),
    )?)?;
    let reg = regex::Regex::new(&master.groups_regex)?;

    // Individual files get filtered against this
    let internal_reg = regex::Regex::new(match master.repack_filter {
        None => "",
        Some(ref filter) => filter,
    })?;

    // Create the zip file, its writer and config
    let zip_file = File::create(zip_name)?;
    let mut zip_writer = zip::ZipWriter::new(zip_file);
    let zip_options = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(9));

    // Create the csv writer
    let mut csv_writer = csv::Writer::from_path(grading_csv_name)?;

    let repack_fn = match (&master.unpack_structure, &master.repack_structure) {
        (Structure::Groups, Structure::Groups) => repack_g2g,
        (Structure::Groups, Structure::Individuals) => repack_g2i,
        _ => todo!(),
    };

    repack_fn(
        &unpacked_path,
        &grading_table,
        &grades,
        &reg,
        &internal_reg,
        &mut zip_writer,
        &zip_options,
        Some(&mut csv_writer),
    )
}

pub fn repack_g2i(
    unpacked_path: &PathBuf,
    grading_table: &[GradingRecord],
    grades: &Grades,
    reg: &regex::Regex,
    internal_reg: &regex::Regex,
    zip_writer: &mut zip::ZipWriter<File>,
    zip_options: &zip::write::FileOptions,
    mut csv_writer: Option<&mut csv::Writer<File>>,
) -> Result<(), Box<dyn Error>> {
    // Start packing stuff
    std::fs::read_dir(unpacked_path)?
        .filter_map(|entry| entry.ok())
        // Check that whatever we're packing is a _directory_
        // and it matches the master regex
        .filter(|entry| {
            entry.path().is_dir() && reg.is_match(entry.file_name().to_str().unwrap_or(""))
        })
        .for_each(|filtered| {
            info!("filtered: {:?}", filtered.file_name());
            let dir_name = filtered.file_name();
            let group_id = dir_name.to_str().unwrap();
            grades
                .collect_students_for_group(grading_table, group_id)
                .iter()
                .for_each(|studi| {
                    // Write the student's record to the csv
                    if let Some(ref mut writer) = csv_writer {
                        writer.serialize(studi).unwrap();
                    }

                    // New directory name. Should be something like
                    // Übungsgruppe AB -- Abgabeteam XY_Name, \
                    // Vorname-12345678_assignsubmission_file_
                    let dir_new_name: String = format!(
                        "{group_id}_{s_name}_{s_id}_assignsubmission_file_",
                        group_id = group_id,
                        s_name = studi.name,
                        s_id = studi.internal_id.strip_prefix("Teilnehmer/in").unwrap()
                    );

                    filtered
                        .path()
                        .read_dir()
                        .unwrap()
                        .filter_map(|f| f.ok())
                        // Match files against the second regex
                        .filter(|f| internal_reg.is_match(f.file_name().to_str().unwrap()))
                        .for_each(|f| {
                            debug!("packing {:?}", f);
                            // Repack each file
                            let fname =
                                format!("{}/{}", dir_new_name, f.file_name().to_str().unwrap());
                            zip_writer.start_file(fname, *zip_options).unwrap();
                            let bytes = std::fs::read(f.path()).unwrap();
                            zip_writer.write_all(&bytes).unwrap();
                        });
                });
        });
    if let Some(writer) = csv_writer {
        writer.flush()?;
    }
    zip_writer.flush()?;

    Ok(())
}

pub fn repack_g2g(
    unpacked_path: &PathBuf,
    grading_table: &[GradingRecord],
    grades: &Grades,
    reg: &regex::Regex,
    internal_reg: &regex::Regex,
    zip_writer: &mut zip::ZipWriter<File>,
    zip_options: &zip::write::FileOptions,
    mut csv_writer: Option<&mut csv::Writer<File>>,
) -> Result<(), Box<dyn Error>> {
    // Start packing stuff
    std::fs::read_dir(unpacked_path)?
        .filter_map(|entry| entry.ok())
        // Check that whatever we're packing is a _directory_
        // and it matches the master regex
        .filter(|entry| {
            entry.path().is_dir() && reg.is_match(entry.file_name().to_str().unwrap_or(""))
        })
        .for_each(|filtered| {
            debug!("filtered: {:?}", filtered.file_name());
            let dir_name = filtered.file_name();
            let group_name = dir_name.to_str().unwrap();
            grades
                .collect_students_for_group(grading_table, group_name)
                .iter()
                .for_each(|studi| {
                    // Write the student's record to the csv
                    if let Some(ref mut writer) = csv_writer {
                        writer.serialize(studi).unwrap();
                    }
                });

            let group_id = grades
                .map
                .iter()
                .find(|m| m.target == group_name)
                .map(|m| m.internal_id.clone().unwrap())
                .unwrap();

            // New directory name. Should be something like
            // Übungsgruppe AB -- Abgabeteam XY_12345678_assignsubmission_file
            let dir_new_name: String = format!("{group_name}_{group_id}_assignsubmission_file");

            filtered
                .path()
                .read_dir()
                .unwrap()
                .filter_map(|f| f.ok())
                // Match files against the second regex
                .filter(|f| internal_reg.is_match(f.file_name().to_str().unwrap()))
                .for_each(|f| {
                    debug!("packing {:?}", f);
                    // Repack each file
                    let fname = format!("{}/{}", dir_new_name, f.file_name().to_str().unwrap());
                    zip_writer.start_file(fname, *zip_options).unwrap();
                    let bytes = std::fs::read(f.path()).unwrap();
                    zip_writer.write_all(&bytes).unwrap();
                });
        });
    if let Some(writer) = csv_writer {
        writer.flush()?;
    }
    zip_writer.flush()?;

    Ok(())
}
