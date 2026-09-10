use std::fmt::Display;
use std::slice::Iter;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum ErrorKind {
    Internal = 1,
    Cli,
    Syntax,
    Type,
    Render,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Internal => write!(f, "internal"),
            ErrorKind::Cli => write!(f, "cli"),
            ErrorKind::Syntax => write!(f, "syntax"),
            ErrorKind::Type => write!(f, "type"),
            ErrorKind::Render => write!(f, "render"),
        }
    }
}

pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: &str) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn error_kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} error: {}", self.kind, self.message)
    }
}

pub struct MultiError(Vec<Error>);

impl MultiError {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn iter(&self) -> Iter<'_, Error> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, kind: ErrorKind, message: &str) {
        self.0.push(Error::new(kind, message));
    }

    pub fn error_kind(&self) -> Option<ErrorKind> {
        self.0.iter().map(|err| err.kind.clone()).min()
    }
}
