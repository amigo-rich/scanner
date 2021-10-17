use crate::manifest::Id;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Operation {
    Compare(Id, Id),
    DeleteManifest(Id),
    Index(PathBuf),
    List,
    Scan(Id),
}
