use crate::parse::constant::Delimiter;
use crate::parse::lex::{Lex, LexParseError};

pub fn is_delimiter(c: u8) -> bool {
    match c as char {
        ',' | ';' => true,
        _ => false,
    }
}

pub fn lex_delim(c: u8) -> Result<Option<Lex>, LexParseError> {
    match c as char {
        ',' => Ok(Some(Lex::Delimiter(Delimiter::Comma))),
        ';' => Ok(Some(Lex::Delimiter(Delimiter::Semicolon))),
        _ => Ok(None),
    }
}
