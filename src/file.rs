use crate::model::yaml::{ApiShapeYaml, ApiYaml, ResponseDataYaml, SystemFolder};
use serde_yaml::from_str;
use std::collections::HashMap;
use std::fs;

pub struct ConfigurationFolder {
    folder: String,
}

impl ConfigurationFolder {
    pub fn new(conf_path: String) -> ConfigurationFolder {
        ConfigurationFolder { folder: conf_path }
    }

    pub fn load_systems(&self) -> Vec<SystemFolder> {
        let conf_dir = fs::read_dir(self.folder.clone())
            .unwrap_or_else(|_| panic!("Could not read configuration folder '{}'", self.folder));

        //
        let system_folders: Vec<SystemFolder> = conf_dir
            // Just keep directories
            .filter_map(|e| e.ok())
            .filter(|entity| entity.metadata().map(|m| m.is_dir()).unwrap_or(false))
            .map(|system_path| {
                let system_name = system_path
                    .path()
                    .file_name()
                    .unwrap_or_else(|| panic!("Could not get system name"))
                    .to_os_string()
                    .into_string()
                    .unwrap();

                // Entries loaded per system folder (./config/system/*)
                let system_entries = fs::read_dir(system_path.path())
                    .unwrap_or_else(|_| {
                        panic!("Could not read directory for system '{}'", system_name)
                    })
                    .filter_map(|e| e.ok())
                    .collect::<Vec<_>>();

                // data directory of the current system folder (./config/system/data/)
                let system_data_dir = system_entries
                    .iter()
                    // Keeps files only
                    .filter(|entity| {
                        entity.file_name() == "data"
                            && entity.metadata().map(|m| m.is_dir()).unwrap_or(false)
                    })
                    .collect::<Vec<_>>();

                // Content of data/*.yml
                let system_data_files: HashMap<String, ResponseDataYaml> = system_data_dir
                    .first()
                    .map(|data_path| {
                        fs::read_dir(data_path.path())
                            .unwrap_or_else(|_| {
                                panic!("Could not read data directory for system '{}'", system_name)
                            })
                            .into_iter()
                            // Keeps files only
                            .filter_map(|i| i.ok())
                            .filter(|entity| {
                                entity.metadata().map(|m| m.is_file()).unwrap_or(false)
                            })
                            .map(|entity| {
                                let filename = entity.file_name().into_string().unwrap();

                                let file_content = fs::read_to_string(entity.path())
                                    .unwrap_or_else(|_| {
                                        panic!("Could not read file '{}'", filename)
                                    });

                                let yaml_response_data_file_content: ResponseDataYaml =
                                    from_str(&*file_content).unwrap_or_else(|_| {
                                        panic!(
                                            "Could not decode response data yaml file '{}'",
                                            filename
                                        )
                                    });

                                // Skip .yml suffix
                                let truncated_filename = &filename[..(filename.len() - 4)];

                                (
                                    truncated_filename.to_string(),
                                    yaml_response_data_file_content,
                                )
                            })
                            .collect()
                    })
                    .unwrap_or(HashMap::new());

                let system_files: Vec<(String, String)> = system_entries
                    .iter()
                    // Keeps files only
                    .filter(|entity| entity.metadata().map(|m| m.is_file()).unwrap_or(false))
                    .map(|entity| {
                        let filename = entity.file_name().into_string().unwrap();

                        (
                            filename.clone(),
                            fs::read_to_string(entity.path()).unwrap_or_else(|_| {
                                panic!("Could not read file '{}'", filename.clone())
                            }),
                        )
                    })
                    .collect();

                let apis: Vec<ApiYaml> = system_files
                    .iter()
                    .filter_map(|(_filename, filecontent)| {
                        let r: serde_yaml::Result<ApiYaml> = from_str(filecontent);
                        r.ok()
                    })
                    .collect();

                let shapes: Vec<ApiShapeYaml> = system_files
                    .iter()
                    .filter_map(|(_filename, filecontent)| {
                        let r: serde_yaml::Result<ApiShapeYaml> = from_str(filecontent);
                        r.ok()
                    })
                    .collect();

                SystemFolder {
                    name: system_name,
                    apis,
                    shapes,
                    data: system_data_files,
                }
            })
            .collect();

        system_folders
    }
}
