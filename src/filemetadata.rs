use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::error::Error;

const READ_MAX: usize = 4098 * 1024;

#[derive(Debug)]
pub struct FileMetadata {
    path: PathBuf,
    hash: String,
    created: time::OffsetDateTime,
    modified: time::OffsetDateTime,
    accessed: time::OffsetDateTime,
}

impl FileMetadata {
    pub fn from_pathbuf(path: PathBuf) -> Result<Self, Error> {
        let file = fs::File::open(&path)?;
        let hash = FileMetadata::calculate_hash(&file)?;
        let (created, modified, accessed) = FileMetadata::times(&file)?;
        Ok(FileMetadata {
            path,
            hash,
            created,
            modified,
            accessed,
        })
    }
    pub fn from_database(
        path: String,
        hash: String,
        created: time::OffsetDateTime,
        modified: time::OffsetDateTime,
        accessed: time::OffsetDateTime,
    ) -> Result<Self, Error> {
        Ok(FileMetadata {
            path: Path::new(&path).to_path_buf(),
            hash,
            created,
            modified,
            accessed,
        })
    }
    fn calculate_hash(file: &fs::File) -> Result<String, Error> {
        let mut reader = BufReader::with_capacity(READ_MAX, file);

        let mut hasher = blake3::Hasher::new();
        loop {
            let buffer = reader.fill_buf()?;
            if buffer.is_empty() {
                break;
            }
            hasher.update(buffer);
            let len = buffer.len();
            reader.consume(len);
        }
        Ok(hasher.finalize().to_string())
    }
    fn times(
        file: &fs::File,
    ) -> Result<
        (
            time::OffsetDateTime,
            time::OffsetDateTime,
            time::OffsetDateTime,
        ),
        Error,
    > {
        let metadata = file.metadata()?;
        let st_created = metadata.created()?;
        let st_modified = metadata.modified()?;
        let st_accessed = metadata.accessed()?;
        Ok((
            time::OffsetDateTime::from(st_created),
            time::OffsetDateTime::from(st_modified),
            time::OffsetDateTime::from(st_accessed),
        ))
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }
    pub fn created(&self) -> &time::OffsetDateTime {
        &self.created
    }
    pub fn modified(&self) -> &time::OffsetDateTime {
        &self.modified
    }
    pub fn accessed(&self) -> &time::OffsetDateTime {
        &self.accessed
    }
}

impl fmt::Display for FileMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Path: {}, Hash: {}, Created: {}, Modified: {}, Accessed: {}",
            self.path.display(),
            self.hash,
            self.created,
            self.modified,
            self.accessed,
        )
    }
}
