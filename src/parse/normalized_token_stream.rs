use std::io::Read;

use crate::parse::error::ParseError;
use crate::parse::token::TokenType::*;
use crate::parse::token::{Scanner, Token};

pub struct NormalizedTokenStream {
    source: Scanner,
    delimiter_stack: Vec<Token>,
}

impl NormalizedTokenStream {
    pub fn new(filename: &str, source: Box<dyn Read>) -> Self {
        Self {
            source: Scanner::new(filename, source),
            delimiter_stack: Vec::new(),
        }
    }
}

fn is_matching(left_delim: &Token, right_delim: &Token) -> bool {
    matches!(
        (&left_delim.0, &right_delim.0),
        (LParen, RParen) | (LBracket, RBracket) | (LBrace, RBrace)
    )
}

fn delimiter_name(delim: &Token) -> char {
    match delim.0 {
        LParen => '(',
        RParen => ')',
        LBracket => '[',
        RBracket => ']',
        LBrace => '{',
        RBrace => '}',
        _ => panic!("not a delimiter: {:?}", delim),
    }
}

impl Iterator for NormalizedTokenStream {
    type Item = Result<Token, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        for mut tok in &mut self.source {
            match &mut tok {
                Ok(left @ Token(LParen | LBracket | LBrace, _)) => {
                    self.delimiter_stack.push(left.clone());
                }
                Ok(right @ Token(RParen | RBracket | RBrace, _)) => {
                    let left = self.delimiter_stack.pop();
                    if !left.is_some_and(|left| is_matching(&left, right)) {
                        return Some(Err(ParseError::Parse {
                            location: right.1.clone(),
                            message: format!("unmatched delimiter ({:?})", delimiter_name(right)),
                        }));
                    }
                }
                Ok(Token(Eof, _)) => {
                    if let Some(left) = self.delimiter_stack.last() {
                        return Some(Err(ParseError::Parse {
                            location: left.1.clone(),
                            message: format!("unmatched delimiter ({:?})", delimiter_name(left)),
                        }));
                    }
                }
                Ok(Token(typ @ Newline, _)) => {
                    if matches!(
                        self.delimiter_stack.last(),
                        Some(Token(LParen | LBracket, _))
                    ) {
                        continue;
                    }
                    *typ = Semicolon;
                }
                _ => {}
            }
            return Some(tok);
        }
        unreachable!()
    }
}
