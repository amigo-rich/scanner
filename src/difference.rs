use crate::filemetadata::FileMetadata;

#[derive(Debug)]
pub enum Type {
    // A file was added
    Add(FileMetadata),
    // A file was removed
    Delete(FileMetadata),
    // A hash mismatch
    Hash(i64, FileMetadata, i64, FileMetadata),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Add(file) => write!(f, "Added: {}", file),
            Type::Delete(file) => write!(f, "Removed: {}", file),
            Type::Hash(manifest_a, file_a, manifest_b, file_b) => write!(
                f,
                "Manifest {}: {}\nManifest {}: {}",
                manifest_a, file_a, manifest_b, file_b,
            ),
        }
    }
}
