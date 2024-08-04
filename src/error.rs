// Importing the formatting module from the standard library.
use std::fmt;
// Importing the IO module from the standard library.
use std::io;
use nom::error::Error;

// Defining the ParseError enum with various variants.
#[derive(Debug)]
pub enum ParseError {
    // An unknown token with the token string and line number.
    UnknownToken { token: String, line: usize },
    // An IO error.
    IoError(io::Error),
    // A syntax error with a message and line number.
    SyntaxError { message: String, line: usize },
    // A Nom error for nom-related parsing errors.
    NomError(nom::Err<Error<&'static str>>),
}

// Implementing the From trait for converting io::Error to ParseError.
impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        ParseError::IoError(err)
    }
}

// Implementing the Display trait for formatting ParseError.
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnknownToken { token, line } => write!(f, "Unknown token '{}' on line {}", token, line),
            ParseError::IoError(err) => write!(f, "IO Error: {}", err),
            ParseError::SyntaxError { message, line } => write!(f, "Syntax error on line {}: {}", line, message),
            ParseError::NomError(err) => write!(f, "Nom Error: {:?}", err),
        }
    }
}

// Implementing the Error trait for ParseError.
impl std::error::Error for ParseError {}
