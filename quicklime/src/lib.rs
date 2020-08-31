pub mod scanner;
pub mod token;

use std::string::String;

#[derive(Debug, PartialEq)]
pub struct Error {
    message: String,
    id: usize,
    kind: ErrorKind,
    index: usize,
    more_info: Option<String>,
    markup: Vec<Markup>,
}

#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    Error,
    Warning,
}

#[derive(Debug, PartialEq)]
pub struct Markup {
    // region of code in question
    index: usize,
    length: usize,
    message: String,
    kind: MarkupKind,
}

#[derive(Debug, PartialEq)]
pub enum MarkupKind {
    Error,
    Warning,
    Note,
}

impl Error {
    pub fn simple_error(message: &str, id: usize, index: usize, length: usize, markup: &str) -> Self {
        Error {
            message: message.to_owned(),
            id,
            index,
            more_info: None,
            kind: ErrorKind::Error,
            markup: vec![ Markup {
                index,
                length,
                message: markup.to_owned(),
                kind: MarkupKind::Error,
            }],
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "/quicklime {}", self.message)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
