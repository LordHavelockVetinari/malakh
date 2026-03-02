use malachite::Integer;
use std::collections::{HashMap, VecDeque};
use std::io::{self, BufReader, Read};
use std::str::FromStr;
use std::sync::LazyLock;

use crate::parse::error::ParseError;
use crate::parse::location::Location;

#[derive(Clone, Debug)]
pub enum TokenType {
    Identifier(String),
    StringLiteral(Vec<u8>),
    Symbol(String),
    Integer(Integer),
    Float(f64),
    Ellipsis,
    Caret,
    Asterisk,
    Slash,
    Percent,
    Plus,
    Minus,
    EqualsEquals,
    BangEquals,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,
    Equals,
    CaretEquals,
    AsteriskEquals,
    SlashEquals,
    PercentEquals,
    PlusEquals,
    MinusEquals,
    RArrow,
    Colon,
    ColonColon,
    ColonEquals,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Semicolon,
    Newline,
    Eof,
    KeywordAnd,
    KeywordAs,
    KeywordBreak,
    KeywordCase,
    KeywordCatch,
    KeywordContinue,
    KeywordDefault,
    KeywordElse,
    KeywordErr,
    KeywordFinally,
    KeywordIf,
    KeywordImport,
    KeywordIn,
    KeywordLoop,
    KeywordModule,
    KeywordNot,
    KeywordOr,
    KeywordOut,
    KeywordStop,
    KeywordSwitch,
    KeywordThrow,
    KeywordTry,
    KeywordWhile,
    KeywordXor,
    InternalKeywordDebug,
}

use TokenType::*;

#[derive(Clone, Debug)]
pub struct Token(pub TokenType, pub Location);

pub struct Scanner {
    location: Location,
    source: io::Bytes<BufReader<Box<dyn Read>>>,
    char_buf: VecDeque<u8>,
}

static KEYWORD_MAP: LazyLock<HashMap<String, TokenType>> = LazyLock::new(|| {
    HashMap::from([
        ("and".to_string(), KeywordAnd),
        ("as".to_string(), KeywordAs),
        ("break".to_string(), KeywordBreak),
        ("case".to_string(), KeywordCase),
        ("catch".to_string(), KeywordCatch),
        ("continue".to_string(), KeywordContinue),
        ("default".to_string(), KeywordDefault),
        ("else".to_string(), KeywordElse),
        ("err".to_string(), KeywordErr),
        ("finally".to_string(), KeywordFinally),
        ("if".to_string(), KeywordIf),
        ("import".to_string(), KeywordImport),
        ("in".to_string(), KeywordIn),
        ("loop".to_string(), KeywordLoop),
        ("module".to_string(), KeywordModule),
        ("not".to_string(), KeywordNot),
        ("or".to_string(), KeywordOr),
        ("out".to_string(), KeywordOut),
        ("stop".to_string(), KeywordStop),
        ("throw".to_string(), KeywordThrow),
        ("try".to_string(), KeywordTry),
        ("switch".to_string(), KeywordSwitch),
        ("while".to_string(), KeywordWhile),
        ("xor".to_string(), KeywordXor),
        ("__debug".to_string(), InternalKeywordDebug),
    ])
});

impl Scanner {
    // source should NOT be a BufReader,
    // as Scanner::new() constructs a BufReader of its own.
    pub fn new(filename: &str, source: Box<dyn Read>) -> Self {
        Self {
            location: Location::from_filename(filename),
            source: BufReader::new(source).bytes(),
            char_buf: VecDeque::new(),
        }
    }

    fn char_at(&mut self, i: usize) -> Result<Option<u8>, ParseError> {
        while self.char_buf.len() <= i {
            match self.source.next() {
                Some(Ok(c)) => self.char_buf.push_back(c),
                Some(Err(err)) => {
                    return Err(ParseError::Io {
                        filename: self.location.filename.clone(),
                        source: err,
                    });
                }
                None => return Ok(None),
            }
        }
        Ok(Some(self.char_buf[i]))
    }

    fn skip(&mut self, i: usize) {
        assert!(
            i <= self.char_buf.len(),
            "cannot skip more characters than buffer size"
        );
        for c in self.char_buf.drain(..i) {
            self.location.advance(c);
        }
    }

    fn skip_get(&mut self, i: usize) -> Location {
        let location = self.location.clone();
        self.skip(i);
        location
    }

    fn char_name(c: u8) -> String {
        if c <= 127 {
            format!("{:?}", c as char)
        } else {
            format!("byte 0x{:02x}", c)
        }
    }

    fn err<T, S: Into<String>>(&self, message: S) -> Result<T, ParseError> {
        Err(ParseError::Token {
            location: self.location.clone(),
            message: message.into(),
        })
    }

    fn is_identifier_character(c: u8) -> bool {
        matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_')
    }

    fn skip_line_comment(&mut self) -> Result<(), ParseError> {
        while self.char_at(0)?.is_some_and(|c| c != b'\n') {
            self.skip(1);
        }
        Ok(())
    }

    fn skip_multiline_comment(&mut self) -> Result<(), ParseError> {
        assert_eq!(self.char_at(0)?, Some(b'/'));
        assert_eq!(self.char_at(1)?, Some(b'*'));
        let location = self.skip_get(2);
        while let (Some(c1), Some(c2)) = (self.char_at(0)?, self.char_at(1)?) {
            if c1 == b'*' && c2 == b'/' {
                self.skip(2);
                return Ok(());
            }
            self.skip(1);
        }
        Err(ParseError::Parse {
            location,
            message: "unterminated comment".to_string(),
        })
    }

    fn skip_whitespace(&mut self) -> Result<(), ParseError> {
        loop {
            if self.char_at(0)?.is_some_and(|c| c == b' ' || c == b'\t') {
                self.skip(1);
            } else if self.char_at(0)? == Some(b'/') && self.char_at(1)? == Some(b'/') {
                self.skip_line_comment()?;
            } else if self.char_at(0)? == Some(b'/') && self.char_at(1)? == Some(b'*') {
                self.skip_multiline_comment()?;
            } else {
                break;
            }
        }
        Ok(())
    }

    fn read_word(&mut self) -> Result<Token, ParseError> {
        let mut ident = String::new();
        let location = self.location.clone();
        while let Some(c) = self.char_at(0)?
            && Self::is_identifier_character(c)
        {
            ident.push(c as char);
            self.skip(1);
        }
        if let Some(token_type) = KEYWORD_MAP.get(&ident) {
            Ok(Token(token_type.clone(), location))
        } else {
            Ok(Token(Identifier(ident), location))
        }
    }

    fn from_hex_digit(digit: u8) -> Option<u8> {
        match digit {
            b'0'..=b'9' => Some(digit - b'0'),
            b'A'..=b'F' => Some(digit - b'A' + 10),
            b'a'..=b'f' => Some(digit - b'a' + 10),
            _ => None,
        }
    }

    fn from_hex_digits(digits: &[u8]) -> u32 {
        debug_assert!(digits.len() <= 8);
        let mut result = 0;
        for &digit in digits {
            debug_assert!(digit < 16);
            result = (result << 4) + digit as u32;
        }
        result
    }

    fn read_escape_sequence(
        &mut self,
        index: &mut usize,
        output: &mut Vec<u8>,
    ) -> Result<(), ParseError> {
        debug_assert_eq!(self.char_at(*index)?, Some(b'\\'));
        *index += 1;
        let Some(c) = self.char_at(*index)? else {
            return self.err("unfinished string literal");
        };
        *index += 1;
        match c {
            b't' => output.push(b'\t'),
            b'r' => output.push(b'\r'),
            b'n' => output.push(b'\n'),
            b'"' => output.push(b'\"'),
            b'\\' => output.push(b'\\'),
            b'x' => {
                if let Some(digit1) = self.char_at(*index)?
                    && let Some(digit2) = self.char_at(*index + 1)?
                    && let Some(digit1) = Self::from_hex_digit(digit1)
                    && let Some(digit2) = Self::from_hex_digit(digit2)
                {
                    *index += 2;
                    output.push(16 * digit1 + digit2);
                } else {
                    self.skip(*index);
                    return self.err("expected 2 hexadecimal digits after `\\x`");
                };
            }
            b'u' | b'U' => {
                let mut digits = [0; 8];
                let len = match c {
                    b'u' => 4,
                    b'U' => 8,
                    _ => unreachable!(),
                };
                for (digit, i) in digits.iter_mut().zip(0..len) {
                    if let Some(d) = self.char_at(*index + i)?
                        && let Some(d) = Self::from_hex_digit(d)
                    {
                        *digit = d;
                    } else {
                        self.skip(*index);
                        return self.err(format!("expected {} hexadecimal digits", len));
                    }
                }
                let codepoint = Self::from_hex_digits(&digits[..len]);
                let Ok(c) = char::try_from(codepoint) else {
                    self.skip(*index);
                    return self.err(format!("invalid unicode codepoint: {:#x}", codepoint));
                };
                let mut buf = [0; 4];
                let s = c.encode_utf8(&mut buf);
                output.extend_from_slice(s.as_bytes());
                *index += len;
            }
            _ => {
                self.skip(*index);
                return self.err("unknown escape sequence");
            }
        }
        Ok(())
    }

    fn read_digits(&mut self) -> Result<(String, Location), ParseError> {
        let mut s = String::new();
        for i in 0usize.. {
            let Some(c) = self.char_at(i)? else {
                break;
            };
            if !c.is_ascii_digit() {
                break;
            }
            s.push(c as char);
        }
        if s.is_empty() {
            self.err("expected one or more digits")?;
        }
        let location = self.skip_get(s.len());
        Ok((s, location))
    }

    fn read_number(&mut self) -> Result<Token, ParseError> {
        let (first_part, location) = self.read_digits()?;
        let second_part = if self.char_at(0)? == Some(b'.') {
            self.skip(1);
            self.read_digits()?.0
        } else {
            String::new()
        };
        let (sign, e_part) = if matches!(self.char_at(0)?, Some(b'E' | b'e')) {
            self.skip(1);
            let sign = match self.char_at(0)? {
                Some(b'+') => {
                    self.skip(1);
                    "+"
                }
                Some(b'-') => {
                    self.skip(1);
                    "-"
                }
                _ => "",
            };
            let e_part = self.read_digits()?.0;
            (sign, e_part)
        } else {
            ("", String::new())
        };
        if let Some(c) = self.char_at(0)?
            && matches!(c, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.')
        {
            return self.err(format!(
                "disallowed character ({}) after number literal",
                Self::char_name(c)
            ));
        }
        if second_part.is_empty() && e_part.is_empty() {
            let n = Integer::from_str(&first_part).unwrap();
            Ok(Token(TokenType::Integer(n), location))
        } else {
            let x = f64::from_str(&format!(
                "{}.{}e{}0{}",
                first_part, second_part, sign, e_part
            ))
            .unwrap();
            Ok(Token(TokenType::Float(x), location))
        }
    }

    fn read_token(&mut self) -> Result<Token, ParseError> {
        self.skip_whitespace()?;
        Ok(match self.char_at(0)? {
            Some(b'.') => {
                if self
                    .char_at(1)?
                    .filter(|&c| Self::is_identifier_character(c) && !c.is_ascii_digit())
                    .is_some()
                {
                    let location = self.skip_get(1);
                    let name = match self.read_word()?.0 {
                        Identifier(name) => name,
                        _ => return self.err("invalid symbol name"),
                    };
                    Token(Symbol(name), location)
                } else if self.char_at(1)? == Some(b'.') {
                    if self.char_at(2)? == Some(b'.') {
                        let location = self.skip_get(3);
                        Token(Ellipsis, location)
                    } else {
                        return self.err("expected an ellipsis ('...'); got only two dots ('..')");
                    }
                } else {
                    return self.err("unexpected character after dot ('.')");
                }
            }
            Some(b':') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(ColonEquals, self.skip_get(2))
                } else if self.char_at(1)? == Some(b':') {
                    Token(ColonColon, self.skip_get(2))
                } else {
                    Token(Colon, self.skip_get(1))
                }
            }
            Some(b'^') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(CaretEquals, self.skip_get(2))
                } else {
                    Token(Caret, self.skip_get(1))
                }
            }
            Some(b'*') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(AsteriskEquals, self.skip_get(2))
                } else {
                    Token(Asterisk, self.skip_get(1))
                }
            }
            Some(b'/') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(SlashEquals, self.skip_get(2))
                } else {
                    Token(Slash, self.skip_get(1))
                }
            }
            Some(b'%') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(PercentEquals, self.skip_get(2))
                } else {
                    Token(Percent, self.skip_get(1))
                }
            }
            Some(b'-') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(MinusEquals, self.skip_get(2))
                } else if self.char_at(1)? == Some(b'>') {
                    Token(RArrow, self.skip_get(2))
                } else {
                    Token(Minus, self.skip_get(1))
                }
            }
            Some(b'+') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(PlusEquals, self.skip_get(2))
                } else {
                    Token(Plus, self.skip_get(1))
                }
            }
            Some(b'=') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(EqualsEquals, self.skip_get(2))
                } else {
                    Token(Equals, self.skip_get(1))
                }
            }
            Some(b'!') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(BangEquals, self.skip_get(2))
                } else {
                    return self.err("unexpected '!' character (did you mean: `not`?)");
                }
            }
            Some(b'<') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(LessEquals, self.skip_get(2))
                } else {
                    Token(Less, self.skip_get(1))
                }
            }
            Some(b'>') => {
                if self.char_at(1)? == Some(b'=') {
                    Token(GreaterEquals, self.skip_get(2))
                } else {
                    Token(Greater, self.skip_get(1))
                }
            }
            Some(b'(') => Token(LParen, self.skip_get(1)),
            Some(b')') => Token(RParen, self.skip_get(1)),
            Some(b'[') => Token(LBracket, self.skip_get(1)),
            Some(b']') => Token(RBracket, self.skip_get(1)),
            Some(b'{') => Token(LBrace, self.skip_get(1)),
            Some(b'}') => Token(RBrace, self.skip_get(1)),
            Some(b',') => Token(Comma, self.skip_get(1)),
            Some(b';') => Token(Semicolon, self.skip_get(1)),
            Some(b'\n') => Token(Newline, self.skip_get(1)),
            Some(b'\r') => {
                if self.char_at(1)? != Some(b'\n') {
                    self.err("carriage return without line feed")?;
                }
                Token(Newline, self.skip_get(2))
            }
            Some(b'"') => {
                let mut s = Vec::new();
                let mut i = 1;
                loop {
                    match self.char_at(i)? {
                        None => return self.err("unfinished string literal"),
                        Some(b'"') => {
                            i += 1;
                            break;
                        }
                        Some(b'\\') => self.read_escape_sequence(&mut i, &mut s)?,
                        Some(b'\n') => {
                            return self.err("a string literal may not contain newlines");
                        }
                        Some(c) => {
                            s.push(c);
                            i += 1;
                        }
                    }
                }
                let location = self.skip_get(i);
                Token(StringLiteral(s), location)
            }
            // "$scanner_reset_location\n" is a special directive that causes the scanner
            // to reset both the line number and column number to 1.
            // It is used by the "-c" argument.
            Some(b'$') => {
                let expected_next = b"$scanner_reset_location\n";
                self.char_at(expected_next.len())?;
                if !self.char_buf.make_contiguous().starts_with(expected_next) {
                    return self.err("unexpected '$' character");
                }
                let location = self.skip_get(expected_next.len());
                self.location.line = 1;
                self.location.column = 1;
                Token(Newline, location)
            }
            Some(c) if c.is_ascii_digit() => self.read_number()?,
            Some(c) if Self::is_identifier_character(c) => self.read_word()?,
            Some(c) => self.err(format!("unrecognized character ({})", Self::char_name(c)))?,
            None => return Ok(Token(Eof, self.location.clone())),
        })
    }
}

impl Iterator for Scanner {
    type Item = Result<Token, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.read_token())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;
    use std::io::Cursor;

    #[test]
    fn test1() {
        let code = br"
            Foo -> {}
            Bar -> {
                baz
                123 quux
            }
        ";
        let tokens = Cursor::new(code.to_vec());
        let mut scanner = Scanner::new("myfile", Box::new(tokens));
        assert_matches!(scanner.next(), Some(Ok(Token(Newline, _))));
        assert_matches!(
            scanner.next(),
            Some(Ok(Token(Identifier(s), _))) if s == "Foo"
        );
        assert_matches!(scanner.next(), Some(Ok(Token(RArrow, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(LBrace, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(RBrace, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(Newline, _))));
        assert_matches!(
            scanner.next(),
            Some(Ok(Token(Identifier(s), _))) if s == "Bar"
        );
        assert_matches!(scanner.next(), Some(Ok(Token(RArrow, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(LBrace, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(Newline, _))));
        assert_matches!(
            scanner.next(),
            Some(Ok(Token(Identifier(s), _))) if s == "baz"
        );
        assert_matches!(scanner.next(), Some(Ok(Token(Newline, _))));
        assert_matches!(
            scanner.next(),
            Some(Ok(Token(Integer(n), _))) if n == 123
        );
        assert_matches!(
            scanner.next(),
            Some(Ok(Token(Identifier(s), _))) if s == "quux"
        );
        assert_matches!(scanner.next(), Some(Ok(Token(Newline, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(RBrace, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(Newline, _))));
        assert_matches!(scanner.next(), Some(Ok(Token(Eof, _))));
    }
}
