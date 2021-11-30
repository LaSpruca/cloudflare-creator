use std::fmt::Display;

#[derive(Debug)]
pub enum ErrorKind {
    IOError,
    IOErrorNonFatal,
    CompilerError,
    SSHError,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {}",
            match self.kind {
                ErrorKind::IOError => "Error during IO operation",
                ErrorKind::CompilerError => "Error during script compilation",
                ErrorKind::SSHError => "Error during SSH",
                ErrorKind::IOErrorNonFatal => "Non fatal error during IO operation",
            },
            self.message
        )
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
