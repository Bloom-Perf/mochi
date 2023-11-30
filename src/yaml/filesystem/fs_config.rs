use crate::yaml::filesystem::fs_system::FsSystem;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct FsConfig {
    folder: PathBuf,
}

impl FsConfig {
    pub fn new(folder: PathBuf) -> FsConfig {
        FsConfig { folder }
    }

    pub fn iter_systems(&self) -> Result<Vec<FsSystem>> {
        let conf_dir = fs::read_dir(self.folder.clone()).context(format!(
            "Could not read configuration folder '{}'",
            self.folder.display().to_string()
        ))?;

        Ok(conf_dir
            // Just keep directories
            .filter_map(|e| e.ok())
            .filter(|entity| entity.metadata().map(|m| m.is_dir()).unwrap_or(false))
            .map(|system_path| FsSystem::new(system_path.path()))
            .collect())
    }
}
