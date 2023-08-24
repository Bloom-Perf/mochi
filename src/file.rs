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
            .unwrap_or_else(|_| panic!("Configuration folder ’{}’ not accessible", self.folder));

        let system_folders: Vec<SystemFolder> = conf_dir
            // Just keep directories
            .filter_map(|e| e.ok())
            .filter(|entity| entity.metadata().map(|m| m.is_dir()).unwrap_or(false))
            .map(|system_path| {
                let system_dir = fs::read_dir(system_path.path())
                    .unwrap_or_else(|_| panic!("Could not read system directory"));

                let system_entries = system_dir.filter_map(|e| e.ok()).collect::<Vec<_>>();

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
                            .unwrap_or_else(|_| panic!("Could not read system directory"))
                            .into_iter()
                            // Keeps files only
                            .filter_map(|i| i.ok())
                            .filter(|entity| {
                                entity.metadata().map(|m| m.is_file()).unwrap_or(false)
                            })
                            .filter_map(|entity| {
                                let file_content = fs::read_to_string(entity.path())
                                    .unwrap_or_else(|_| {
                                        panic!(
                                            "File ’{}’ could not be read",
                                            entity.file_name().into_string().unwrap()
                                        )
                                    });

                                dbg!(file_content.clone());

                                let r: serde_yaml::Result<ResponseDataYaml> =
                                    from_str(&*file_content);

                                let filename = entity.file_name().into_string().unwrap();
                                let truncated_filename = &filename[..(filename.len() - 4)];
                                dbg!(truncated_filename);
                                r.ok().map(|file| (truncated_filename.to_string(), file))
                            })
                            .collect()
                    })
                    .unwrap_or(HashMap::new());

                let system_files: Vec<String> = system_entries
                    .iter()
                    // Keeps files only
                    .filter(|entity| entity.metadata().map(|m| m.is_file()).unwrap_or(false))
                    .map(|entity| {
                        fs::read_to_string(entity.path()).unwrap_or_else(|_| {
                            panic!(
                                "File ’{}’ could not be read",
                                entity.file_name().into_string().unwrap()
                            )
                        })
                    })
                    .collect();

                let apis: Vec<ApiYaml> = system_files
                    .iter()
                    .filter_map(|f| {
                        let r: serde_yaml::Result<ApiYaml> = from_str(f);
                        dbg!(r);
                        let r: serde_yaml::Result<ApiYaml> = from_str(f);
                        r.ok()
                    })
                    .collect();

                let shapes: Vec<ApiShapeYaml> = system_files
                    .iter()
                    .filter_map(|f| {
                        let r: serde_yaml::Result<ApiShapeYaml> = from_str(f);
                        r.ok()
                    })
                    .collect();

                SystemFolder {
                    name: system_path.file_name().into_string().unwrap(),
                    apis,
                    shapes,
                    data: system_data_files,
                }
            })
            .collect();

        system_folders
    }
}
