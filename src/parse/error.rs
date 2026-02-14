use std::io;
use std::rc::Rc;

use crate::parse::location::Location;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("IO error while reading source file {filename}: {source}")]
    Io {
        filename: Rc<String>,
        #[source]
        source: io::Error,
    },
    #[error("parse error at {location}: {message}")]
    Token { location: Location, message: String },
    #[error("parse error at {location}: {message}")]
    Parse { location: Location, message: String },
}
