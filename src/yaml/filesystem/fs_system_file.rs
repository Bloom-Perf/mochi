use anyhow::{Context, Result};
use log::debug;
use std::fs;
use std::path::PathBuf;

pub struct FsSystemFile {
    pub path: PathBuf,
    pub content: String,
}

impl FsSystemFile {
    pub fn from(path: PathBuf) -> Result<FsSystemFile> {
        debug!("Reading system file '{}'", path.display());
        Ok(FsSystemFile {
            path: path.clone(),
            content: fs::read_to_string(&path)
                .context(format!("Could not read data file '{}'", path.display()))?,
        })
    }
}
