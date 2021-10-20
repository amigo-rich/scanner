#[derive(Debug)]
pub enum SchemaFileProblem {
    NoComponents,
    WrongNumberOfComponents,
    InvalidUTF8,
    InvalidPath,
}

impl std::fmt::Display for SchemaFileProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error_description = match self {
            SchemaFileProblem::NoComponents => {
                "Splitting the schema filename resulted in no components"
            }
            SchemaFileProblem::WrongNumberOfComponents => {
                "Splitting the schema filename result in an invalid number of components"
            }
            SchemaFileProblem::InvalidUTF8 => {
                "While converting the Path to a String, invalid utf8 was encountered"
            }
            SchemaFileProblem::InvalidPath => "The provided Path was invalid",
        };
        write!(f, "{}", error_description)
    }
}

#[derive(Debug)]
pub enum Error {
    EmptyString,
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
        let error_description = match self {
            Error::EmptyString => String::from("An empty string was provided"),
            Error::InvalidSchemaDirectory(path) => {
                format!("The provided pathbuf: {:?} is invalid", path)
            }
            Error::InvalidSchemaFile(problem) => {
                format!("A provided schema file is invalid: {}", problem)
            }
            Error::IO(e) => format!("An IO Error occurred: {}", e),
            Error::NoFile(path) => format!("The file provided does not exist: {:?}", path),
            Error::ParseInt(e) => format!(
                "While parsing a String to an Integer, an error occured: {}",
                e
            ),
            Error::NoSchemaFile(path) => format!("No schema files found at: {:?}", path),
            Error::Rusqlite(e) => format!("A rusqlite error occurred: {}", e),
            Error::SendPathBuf(e) => format!("A Send error occurred: {}", e),
            Error::SendPathBufHash(e) => format!("A send derror occurred: {}", e),
            Error::ThreadJoin => String::from("An error occurred from a thread"),
        };
        write!(f, "{}", error_description)
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
