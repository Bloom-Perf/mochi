use std::fs;
use serde_yaml::from_str;
use crate::model::yaml::{ApiShapeYaml, ApiYaml, SystemFolder};

pub struct ConfigurationFolder {
    folder: &'static str
}

impl ConfigurationFolder {
    pub fn new(conf_path: &'static str) -> ConfigurationFolder {
        ConfigurationFolder {
            folder: conf_path
        }
    }

    pub fn load_systems(&self) -> Vec<SystemFolder> {
        let conf_dir = fs::read_dir(self.folder)
            .expect(&*format!("Configuration folder ’{}’ not accessible", self.folder));

        let system_folders: Vec<SystemFolder> = conf_dir
            // Just keep directories
            .filter_map(|e| e.ok())
            .filter(|entity| entity.metadata().map(|m| m.is_dir()).unwrap_or(false))
            .map(|system_path| {
                let system_dir = fs::read_dir(system_path.path())
                    .expect(&*format!("Could not read system directory"));

                let system_files: Vec<String> = system_dir
                    .filter_map(|e| e.ok())
                    .collect::<Vec<_>>()
                    .into_iter()
                    // Keeps files only
                    .filter(|entity| entity.metadata().map(|m| m.is_file()).unwrap_or(false))
                    .map(|entity|
                        fs::read_to_string(entity.path())
                            .expect(&*format!("File ’{}’ could not be read", entity.file_name().into_string().unwrap()))
                    ).collect();

                let apis: Vec<ApiYaml> = system_files.iter().filter_map(|f| {
                    let r: serde_yaml::Result<ApiYaml> = from_str(f);
                    r.ok()
                }).collect();
                let shapes: Vec<ApiShapeYaml> = system_files.iter().filter_map(|f| {
                    let r: serde_yaml::Result<ApiShapeYaml> = from_str(f);
                    r.ok()
                }).collect();

                SystemFolder {
                    name: system_path.file_name().into_string().unwrap(),
                    apis,
                    shapes
                }
            })
            .collect();

        system_folders
    }
}