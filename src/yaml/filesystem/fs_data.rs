use crate::yaml::filesystem::fs_data_file::FsDataFile;
use anyhow::Result;
use log::debug;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct FsData {
    pub path: PathBuf,
}

impl FsData {
    pub const FOLDER: &'static str = "data";
    pub fn new(path: PathBuf) -> FsData {
        FsData { path }
    }

    pub fn iter_files(&self) -> Result<Vec<FsDataFile>> {
        debug!(
            "Iterating over files of data folder '{}'",
            self.path.display()
        );
        WalkDir::new(self.path.clone())
            .into_iter()
            // Keeps files only
            .filter_map(|i| i.ok())
            .filter(|entity| entity.metadata().map(|m| m.is_file()).unwrap_or(false))
            .map(|entity| FsDataFile::from(entity.path().to_path_buf()))
            .collect()
    }
}
