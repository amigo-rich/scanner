use std::path::PathBuf;

#[derive(Debug)]
pub enum Operation {
    Compare(i64, i64),
    DeleteManifest(i64),
    Index(PathBuf),
    List,
    Scan(i64),
}
