use crate::yaml::filesystem::fs_data_file::FsDataFile;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct FsData {
    pub folder: PathBuf,
}

impl FsData {
    pub fn new(path: PathBuf) -> FsData {
        FsData { folder: path }
    }

    pub fn iter_files(&self) -> Result<Vec<FsDataFile>> {
        fs::read_dir(self.folder.clone())
            .context(format!(
                "Could not read data directory for system '{}'",
                self.folder.display()
            ))?
            // Keeps files only
            .filter_map(|i| i.ok())
            .filter(|entity| entity.metadata().map(|m| m.is_file()).unwrap_or(false))
            .map(|entity| {
                let path = entity.path();

                Ok(FsDataFile::new(
                    path.clone(),
                    fs::read_to_string(entity.path())
                        .context(format!("Could not read data file '{}'", path.display()))?,
                ))
            })
            .collect()
    }
}
