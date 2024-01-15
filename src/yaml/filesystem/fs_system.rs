use crate::yaml::filesystem::fs_api::FsApi;
use crate::yaml::filesystem::fs_data::FsData;
use crate::yaml::filesystem::fs_system_file::FsSystemFile;
use anyhow::{Context, Result};
use log::{debug, warn};
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

pub struct FsSystem {
    pub path: PathBuf,
}

impl FsSystem {
    pub fn new(path: PathBuf) -> FsSystem {
        FsSystem { path }
    }

    // Entries loaded per system FOLDER (./config/system/*)
    fn get_entries(&self) -> Result<Vec<DirEntry>> {
        debug!(
            "Iterating over entries of system folder '{}'",
            self.path.display()
        );

        Ok(fs::read_dir(self.path.clone())
            .context(format!(
                "Could not read directory for system '{}'",
                self.path.display()
            ))?
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>())
    }

    pub fn get_data_folder(&self) -> Result<Option<FsData>> {
        debug!(
            "Fetching {} folder of system folder '{}'",
            FsData::FOLDER,
            self.path.display()
        );

        // data directory of the current system FOLDER (./config/system/data/)
        let fs_data = self
            .get_entries()?
            .iter()
            // Keeps files only
            .find(|entity| {
                entity.file_name() == FsData::FOLDER
                    && entity.metadata().map(|m| m.is_dir()).unwrap_or(false)
            })
            .map(|dir_entry| FsData::new(dir_entry.path()));

        if fs_data.is_none() {
            warn!("No data FOLDER found for system '{}'", self.path.display());
        }

        Ok(fs_data)
    }

    pub fn iter_files(&self) -> Result<Vec<FsSystemFile>> {
        debug!(
            "Iterating over files of system folder '{}'",
            self.path.display()
        );
        self.get_entries()?
            .iter()
            // Keeps files only
            .filter(|entity| entity.metadata().map(|m| m.is_file()).unwrap_or(false))
            .map(|entity| FsSystemFile::from(entity.path()))
            .collect()
    }

    pub fn iter_api_folders(&self) -> Result<Vec<FsApi>> {
        debug!(
            "Iterating over api folders of system folder '{}'",
            self.path.display()
        );

        Ok(self
            .get_entries()?
            .iter()
            // Keeps files only
            .filter(|entity| {
                entity
                    .metadata()
                    .map(|m| m.is_dir() && !entity.file_name().eq(FsData::FOLDER))
                    .unwrap_or(false)
            })
            .map(|entity| FsApi::new(entity.path()))
            .collect())
    }
}
