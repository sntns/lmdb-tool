use error_stack::Context;

#[derive(Debug)]
pub enum Error {
    ReadError,
    WriteError,
    LockError,
    UnsupportedFormat,
    InvalidFileFormat,
    VersionNotSupported,
    NoReader,
    IoError(std::io::Error),
}

impl Context for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ReadError => write!(f, "Read error"),
            Error::WriteError => write!(f, "Write error"),
            Error::LockError => write!(f, "Lock error"),
            Error::UnsupportedFormat => write!(f, "Unsupported format"),
            Error::InvalidFileFormat => write!(f, "Invalid file format"),
            Error::VersionNotSupported => write!(f, "Version not supported"),
            Error::NoReader => write!(f, "No reader"),
            Error::IoError(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

