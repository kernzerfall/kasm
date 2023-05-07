use crate::config::*;
use std::{error::Error, fs::*, path::PathBuf};

pub fn init_master(cfg: &MasterCfg) -> Result<(), Box<dyn Error>> {
    let cfg_path: PathBuf = MASTER_CFG_FILENAME.into();
    if cfg_path.is_file() {
        return Err(format!("{} already exists!", MASTER_CFG_FILENAME).into());
    }

    write(cfg_path, toml::to_string_pretty(cfg)?)?;

    Ok(())
}
