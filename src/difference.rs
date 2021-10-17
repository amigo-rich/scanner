pub enum Type {
    // A file was added
    Add(String, String),
    // A file was removed
    Delete(String, String),
    // A hash mismatch
    Hash(i64, String, String, i64, String, String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Add(path, hash) => write!(f, "Added: {:?}, {}", path, hash),
            Type::Delete(path, hash) => write!(f, "Removed: {:?}, {}", path, hash),
            Type::Hash(manifest_a, path_a, hash_a, manifest_b, path_b, hash_b) => write!(
                f,
                "Modified: Manifest {}: {:?} {}, Manifest {}: {:?}, {}",
                manifest_a, path_a, hash_a, manifest_b, path_b, hash_b
            ),
        }
    }
}
