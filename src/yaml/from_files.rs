use crate::yaml::filesystem::fs_api::FsApi;
use crate::yaml::filesystem::fs_config::FsConfig;
use crate::yaml::filesystem::fs_data::FsData;
use crate::yaml::filesystem::fs_data_file::FsDataFile;
use crate::yaml::filesystem::fs_system::FsSystem;
use crate::yaml::{
    ApiFolder, ApiShapeYaml, ApiYaml, ConfFolder, ProxyYaml, ResponseDataYaml, SystemFolder,
};
use anyhow::{Context, Result};
use log::{debug, error};
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
        debug!("Loading data file '{}'", fs_data_file.path.display());

        let filename_key = fs_data_file
            .path
            .with_extension("") // Skip .yml suffix
            .iter()
            .skip_while(|p| !p.eq_ignore_ascii_case(FsData::FOLDER))
            .skip(1)
            .collect::<Vec<&OsStr>>()
            .join("/".as_ref())
            .into_string()
            .unwrap();

        let file_content = fs::read_to_string(&fs_data_file.path).context(format!(
            "Could not read data file '{}'",
            &fs_data_file.path.display()
        ))?;

        let yaml_response_data_file_content: ResponseDataYaml =
            from_str(&file_content).context(format!(
                "Could not decode response data yaml file '{}'",
                &fs_data_file.path.display()
            ))?;

        Ok((filename_key, yaml_response_data_file_content))
    }
    pub(self) fn load_fs_data(fs_data: FsData) -> Result<HashMap<String, ResponseDataYaml>> {
        debug!("Loading data folder '{}'", fs_data.path.display());

        fs_data
            .iter_files()?
            .into_iter()
            .map(ConfigurationFolder::load_fs_data_file)
            .collect()
    }

    pub(self) fn load_fs_api_folder(fs_api: FsApi) -> Result<ApiFolder> {
        debug!("Loading api folder '{}'", fs_api.path.display());

        let data = match fs_api.get_data_folder()? {
            Some(fs_data) => ConfigurationFolder::load_fs_data(fs_data)?,
            None => HashMap::new(),
        };

        let apis: Vec<ApiYaml> = fs_api
            .iter_api_files()?
            .into_iter()
            .filter_map(|file| -> Option<ApiYaml> {
                from_str(&file.content)
                    .context(format!(
                        "Failed to decode api '{}' in api folder '{}'",
                        file.path.display(),
                        fs_api.path.display()
                    ))
                    .map_err(|e| error!("{:?}", e))
                    .ok()
            })
            .collect();

        let shape: Option<ApiShapeYaml> =
            fs_api
                .iter_shape_files()?
                .into_iter()
                .find_map(|file| -> Option<ApiShapeYaml> {
                    from_str(&file.content)
                        .context(format!(
                            "Failed to decode api shape '{}' in api folder '{}'",
                            file.path.display(),
                            fs_api.path.display()
                        ))
                        .map_err(|e| error!("{:?}", e))
                        .ok()
                });

        let proxy: Option<ProxyYaml> =
            fs_api
                .iter_proxy_files()?
                .into_iter()
                .find_map(|file| -> Option<ProxyYaml> {
                    from_str(&file.content)
                        .context(format!(
                            "Failed to decode api proxy '{}' in api folder '{}'",
                            file.path.display(),
                            fs_api.path.display()
                        ))
                        .map_err(|e| error!("{:?}", e))
                        .ok()
                });

        Ok(ApiFolder {
            name: fs_api
                .path
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap(),
            apis,
            shape,
            proxy,
            data,
        })
    }

    pub(self) fn load_fs_system(fs_system: FsSystem) -> Result<SystemFolder> {
        let system_path = &fs_system.path.display();
        debug!("Loading system folder '{}'", system_path);

        let data = match fs_system.get_data_folder()? {
            Some(fs_data) => ConfigurationFolder::load_fs_data(fs_data)?,
            None => HashMap::new(),
        };

        let api_folders: Vec<ApiFolder> = fs_system
            .iter_api_folders()?
            .into_iter()
            .filter_map(|fs_api| {
                let fs_api_path = fs_api.path.display().to_string();
                ConfigurationFolder::load_fs_api_folder(fs_api)
                    .context(format!(
                        "Failed to decode api folder '{}' in system folder '{}'",
                        fs_api_path, system_path
                    ))
                    .map_err(|e| error!("{:?}", e))
                    .ok()
            })
            .collect();

        let apis: Vec<ApiYaml> = fs_system
            .iter_api_files()?
            .into_iter()
            .filter_map(|file| -> Option<ApiYaml> {
                from_str(&file.content)
                    .context(format!(
                        "Failed to decode api file '{}' in system folder '{}'",
                        file.path.display(),
                        system_path
                    ))
                    .map_err(|e| error!("{:?}", e))
                    .ok()
            })
            .collect();

        let shape: Option<ApiShapeYaml> =
            fs_system
                .iter_shape_files()?
                .into_iter()
                .find_map(|file| -> Option<ApiShapeYaml> {
                    from_str(&file.content)
                        .context(format!(
                            "Failed to decode shape file '{}' in system folder '{}'",
                            file.path.display(),
                            system_path
                        ))
                        .map_err(|e| error!("{:?}", e))
                        .ok()
                });
        let proxy: Option<ProxyYaml> =
            fs_system
                .iter_proxy_files()?
                .into_iter()
                .find_map(|file| -> Option<ProxyYaml> {
                    from_str(&file.content)
                        .context(format!(
                            "Failed to decode api proxy '{}' in api folder '{}'",
                            file.path.display(),
                            fs_system.path.display()
                        ))
                        .map_err(|e| error!("{:?}", e))
                        .ok()
                });

        Ok(SystemFolder {
            name: fs_system
                .path
                .file_name()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap(),
            api_folders,
            apis,
            shape,
            proxy,
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
                .filter_map(|system| {
                    let path = system.path.display().to_string();
                    ConfigurationFolder::load_fs_system(system)
                        .context(format!("Loading fs filesystem '{}' in config", &path))
                        .map_err(|err| error!("{:?}", err))
                        .ok()
                })
                .collect(),
        })
    }
}
