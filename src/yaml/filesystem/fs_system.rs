use crate::yaml::filesystem::fs_data::FsData;
use crate::yaml::filesystem::fs_system_file::FsSystemFile;
use anyhow::{Context, Result};
use log::warn;
use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;

pub struct FsSystem {
    pub folder: PathBuf,
}

impl FsSystem {
    pub fn new(system_path: PathBuf) -> FsSystem {
        FsSystem {
            folder: system_path,
        }
    }

    // Entries loaded per system folder (./config/system/*)
    fn get_entries(&self) -> Result<Vec<DirEntry>> {
        Ok(fs::read_dir(self.folder.clone())
            .context(format!(
                "Could not read directory for system '{}'",
                self.folder.display().to_string()
            ))?
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>())
    }

    pub fn get_data_folder(&self) -> Result<Option<FsData>> {
        // data directory of the current system folder (./config/system/data/)
        let fs_data = self
            .get_entries()?
            .iter()
            // Keeps files only
            .find(|entity| {
                entity.file_name() == "data"
                    && entity.metadata().map(|m| m.is_dir()).unwrap_or(false)
            })
            .map(|dir_entry| FsData::new(dir_entry.path()));

        if fs_data.is_none() {
            warn!(
                "No data folder found for system \"{}\"",
                self.folder.display().to_string()
            );
        }

        Ok(fs_data)
    }

    pub fn iter_files(&self) -> Result<Vec<FsSystemFile>> {
        self.get_entries()?
            .iter()
            // Keeps files only
            .filter(|entity| entity.metadata().map(|m| m.is_file()).unwrap_or(false))
            .map(|entity| {
                let path = entity.path();

                Ok(FsSystemFile::new(
                    path.clone(),
                    fs::read_to_string(entity.path()).context(format!(
                        "Could not read system file '{}'",
                        path.display().to_string()
                    ))?,
                ))
            })
            .collect()
    }
}
