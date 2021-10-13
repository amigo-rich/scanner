use chrono::{DateTime, Local};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Operation {
    Index(PathBuf),
    List,
    Scan(i64, PathBuf),
}
