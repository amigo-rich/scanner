use std::path::PathBuf;

pub enum Type {
    // A file was added
    Add(PathBuf, String),
    // A file was removed
    Delete(PathBuf, String),
    // A hash mismatch
    Hash(PathBuf, String, PathBuf, String),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Add(path, hash) => (),
            Type::Delete(path, hash) => (),
            Type::Hash(path_a, hash_a, path_b, hash_b) => (),
        }
    }
}

pub struct Difference {
    difference: Vec<Type>,
}

impl Difference {
    pub fn add(&mut self, difference_type: Type) {
        self.difference.push(difference_type);
    }
}
