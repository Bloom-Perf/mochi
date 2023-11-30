use crate::yaml::filesystem::fs_config::FsConfig;
use crate::yaml::filesystem::fs_data::FsData;
use crate::yaml::filesystem::fs_data_file::FsDataFile;
use crate::yaml::filesystem::fs_system::FsSystem;
use crate::yaml::{ApiShapeYaml, ApiYaml, ConfFolder, ResponseDataYaml, SystemFolder};
use anyhow::{Context, Result};
use serde_yaml::from_str;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

pub struct ConfigurationFolder {
    folder: String,
}

impl ConfigurationFolder {
    pub fn new(path: String) -> ConfigurationFolder {
        ConfigurationFolder { folder: path }
    }
    pub(self) fn load_fs_data_file(fs_data_file: FsDataFile) -> Result<(String, ResponseDataYaml)> {
        let filename_key = fs_data_file
            .path
            .iter()
            .skip_while(|p| !p.eq_ignore_ascii_case(FsData::FOLDER))
            .skip(1)
            .collect::<Vec<&OsStr>>()
            .join("/".as_ref())
            .into_string()
            .unwrap();

        let file_content = fs::read_to_string(fs_data_file.path)
            .context(format!("Could not read data file '{}'", filename_key))?;

        let yaml_response_data_file_content: ResponseDataYaml =
            from_str(&file_content).context(format!(
                "Could not decode response data yaml file '{}'",
                filename_key
            ))?;

        // Skip .yml suffix
        let truncated_filename = &filename_key[..(filename_key.len() - 4)];

        Ok((
            truncated_filename.to_string(),
            yaml_response_data_file_content,
        ))
    }
    pub(self) fn load_fs_data(fs_data: FsData) -> Result<HashMap<String, ResponseDataYaml>> {
        fs_data
            .iter_files()?
            .into_iter()
            .map(ConfigurationFolder::load_fs_data_file)
            .collect()
    }

    pub(self) fn load_fs_system(fs_system: FsSystem) -> Result<SystemFolder> {
        let data = fs_system
            .get_data_folder()?
            .map(ConfigurationFolder::load_fs_data)
            .unwrap_or(Ok(HashMap::new()))?;

        dbg!(data.clone());

        let apis: Vec<ApiYaml> = fs_system
            .iter_files()?
            .into_iter()
            .filter_map(|file| -> Option<ApiYaml> {
                from_str(&file.content)
                    .context(format!("Failed to decode api \"{}\"", file.path.display()))
                    .ok()
            })
            .collect();

        let shapes: Vec<ApiShapeYaml> = fs_system
            .iter_files()?
            .into_iter()
            .filter_map(|file| -> Option<ApiShapeYaml> {
                from_str(&file.content)
                    .context(format!(
                        "Failed to decode api shape \"{}\"",
                        file.path.display()
                    ))
                    .ok()
            })
            .collect();

        Ok(SystemFolder {
            name: fs_system
                .path
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap(),
            apis,
            shapes,
            data,
        })
    }
    pub fn load_from_filesystem(&self) -> Result<ConfFolder> {
        let conf_path = PathBuf::from(self.folder.clone());
        let fs_config = FsConfig::new(conf_path);

        Ok(ConfFolder {
            systems: fs_config
                .iter_systems()?
                .into_iter()
                .filter_map(|system| ConfigurationFolder::load_fs_system(system).ok())
                .collect(),
        })
    }
}
