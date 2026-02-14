use std::fmt::{self, Display};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Location {
    // Rc<String> instead of Rc<str> to minimize pointer size.
    pub filename: Rc<String>,
    pub line: u32,
    pub column: u32,
}

impl Location {
    pub fn from_filename(filename: &str) -> Self {
        Self {
            filename: Rc::new(filename.to_string()),
            line: 1,
            column: 1,
        }
    }

    pub fn advance(&mut self, byte: u8) {
        if byte == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    pub fn is_unknown(&self) -> bool {
        self.line == 0
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.filename, self.line, self.column)
    }
}

impl Default for Location {
    fn default() -> Self {
        thread_local! {
            static DEFAULT_STRING: Rc<String> = Rc::new("unknown".to_string());
        }
        Self {
            filename: DEFAULT_STRING.with(Rc::clone),
            line: 0,
            column: 0,
        }
    }
}
