use chrono::NaiveDateTime;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Operation {
    Index(PathBuf),
    Scan(NaiveDateTime, PathBuf),
}
