use std::path::PathBuf;

#[derive(Debug)]
pub enum Operation {
    DeleteManifest(i64),
    Index(PathBuf),
    List,
    Scan(i64, PathBuf),
}
