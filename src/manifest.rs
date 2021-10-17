use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Id(pub i64);
#[derive(Debug)]
pub struct Timestamp(pub i64);

pub struct Manifest {
    id: i64,
    timestamp: i64,
    file_path: PathBuf,
}

impl Manifest {
    pub fn from_database(id: Id, timestamp: Timestamp, file_path: String) -> Self {
        let file_path = Path::new(&file_path).to_path_buf();
        Manifest {
            id: id.0,
            timestamp: timestamp.0,
            file_path,
        }
    }
    pub fn id(&self) -> Id {
        Id(self.id)
    }
    pub fn timestamp(&self) -> Timestamp {
        Timestamp(self.timestamp)
    }
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

impl std::fmt::Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\t{}\t{}",
            self.id,
            self.timestamp,
            self.file_path.display()
        )
    }
}
