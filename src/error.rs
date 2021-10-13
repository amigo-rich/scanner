#[derive(Debug)]
pub enum SchemaFileProblem {
    NoComponents,
    WrongNumberOfComponents,
    InvalidUTF8,
    InvalidPath,
}

#[derive(Debug)]
pub enum Error {
    InvalidMessage,
    InvalidSchemaDirectory(std::path::PathBuf),
    InvalidSchemaFile(SchemaFileProblem),
    IO(std::io::Error),
    NoFile(std::path::PathBuf),
    ParseInt(std::num::ParseIntError),
    NoSchemaFile(std::path::PathBuf),
    Rusqlite(rusqlite::Error),
    SendPathBuf(std::sync::mpsc::SendError<std::path::PathBuf>),
    SendPathBufHash(std::sync::mpsc::SendError<(std::path::PathBuf, String)>),
    ThreadJoin,
}

impl std::error::Error for Error {}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Rusqlite(e)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}
impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseInt(e)
    }
}

impl From<std::sync::mpsc::SendError<std::path::PathBuf>> for Error {
    fn from(e: std::sync::mpsc::SendError<std::path::PathBuf>) -> Self {
        Error::SendPathBuf(e)
    }
}

impl From<std::sync::mpsc::SendError<(std::path::PathBuf, String)>> for Error {
    fn from(e: std::sync::mpsc::SendError<(std::path::PathBuf, String)>) -> Self {
        Error::SendPathBufHash(e)
    }
}

impl From<Box<dyn std::any::Any + Send + 'static>> for Error {
    fn from(_: Box<dyn std::any::Any + Send + 'static>) -> Self {
        Error::ThreadJoin
    }
}
