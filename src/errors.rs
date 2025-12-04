use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum FileError {
    IOError,
    AllDownloadsFailed,
    DownloadFailed,
    DeleteFailed,
}

impl From<reqwest::Error> for FileError {
    fn from(_: reqwest::Error) -> Self {
        FileError::DownloadFailed
    }
}

impl From<std::io::Error> for FileError {
    fn from(_: std::io::Error) -> Self {
        FileError::IOError
    }
}

impl Display for FileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileError::IOError => write!(f, "IO Error"),
            FileError::AllDownloadsFailed => write!(f, "All Downloads Failed"),
            FileError::DownloadFailed => write!(f, "Download Failed"),
            FileError::DeleteFailed => write!(f, "Delete Failed"),
        }
    }
}
