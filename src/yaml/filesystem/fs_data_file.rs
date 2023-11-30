use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct FsDataFile {
    pub path: PathBuf,
    pub content: String,
}

impl FsDataFile {
    pub fn from(path: PathBuf) -> Result<FsDataFile> {
        Ok(FsDataFile {
            path: path.clone(),
            content: fs::read_to_string(path.clone())
                .context(format!("Could not read data file '{}'", path.display()))?,
        })
    }
}
