use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct FsSystemFile {
    pub path: PathBuf,
    pub content: String,
}

impl FsSystemFile {
    pub fn from(path: PathBuf) -> Result<FsSystemFile> {
        Ok(FsSystemFile {
            path: path.clone(),
            content: fs::read_to_string(path.clone())
                .context(format!("Could not read data file '{}'", path.display()))?,
        })
    }
}
