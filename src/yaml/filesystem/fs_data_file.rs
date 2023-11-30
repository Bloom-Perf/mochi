use std::path::PathBuf;

pub struct FsDataFile {
    pub path: PathBuf,
    pub content: String,
}

impl FsDataFile {
    pub fn new(path: PathBuf, content: String) -> FsDataFile {
        FsDataFile { path, content }
    }
}
