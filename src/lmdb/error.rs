use error_stack::Context;

#[derive(Debug)]
pub enum Error {
    ReadError,
    WriteError,
    InvalidFileFormat,
    VersionNotSupported,
    NoReader,
}

impl Context for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ReadError => write!(f, "Read error"),
            Error::WriteError => write!(f, "Write error"),
            Error::InvalidFileFormat => write!(f, "Invalid file format"),
            Error::VersionNotSupported => write!(f, "Version not supported"),
            Error::NoReader => write!(f, "No reader"),
        }
    }
}

