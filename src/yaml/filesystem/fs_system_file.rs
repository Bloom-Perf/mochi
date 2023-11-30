use std::path::PathBuf;

pub struct FsSystemFile {
    pub path: PathBuf,
    pub content: String,
}

impl FsSystemFile {
    pub fn new(path: PathBuf, content: String) -> FsSystemFile {
        FsSystemFile { path, content }
    }
}
