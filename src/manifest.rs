use std::path::{Path, PathBuf};

pub struct Manifest {
    id: i64,
    timestamp: i64,
    file_path: PathBuf,
}

impl Manifest {
    pub fn from_database(id: i64, timestamp: i64, file_path: String) -> Self {
        let file_path = Path::new(&file_path).to_path_buf();
        Manifest {
            id,
            timestamp,
            file_path,
        }
    }
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}
