use crate::parse::constant::Brace;
use crate::parse::lex::{BraceContext, BraceStatus, Lex, LexParseError};

pub fn lex_brace(
    input: &[u8],
    index: &mut usize,
    acc: BraceContinuation,
    lex_index: &usize,
    context: &mut BraceContext,
) -> Result<Option<Lex>, LexParseError> {
    *index += 1;
    match acc {
        BraceContinuation::Paranthese(status) => {
            let level = context.update_refs(Brace::Brace, status, *index, *lex_index)?;
            return Ok(Some(Lex::Brace(Brace::Brace, status, level, *index)));
        }
        BraceContinuation::SquareBracket(status) => {
            let level = context.update_refs(Brace::Square, status, *index, *lex_index)?;
            return Ok(Some(Lex::Brace(Brace::Square, status, level, *index)));
        }
        BraceContinuation::CurlyBrackets(status) => {
            let level = context.update_refs(Brace::Bracket, status, *index, *lex_index)?;
            return Ok(Some(Lex::Brace(Brace::Bracket, status, level, *index)));
        }
        BraceContinuation::AngleBrackets(status) => {
            let level = context.update_refs(Brace::Angle, status, *index, *lex_index)?;
            return Ok(Some(Lex::Brace(Brace::Angle, status, level, *index)));
        }
        BraceContinuation::Quote => {
            let mut escaped = false;
            let mut c = input[*index];
            let mut s = Vec::new();
            while *index < input.len() && (!escaped && c != 34) {
                if c == 10 {
                    return Err(LexParseError::QuoteNotEnded(*index));
                }
                escaped = match c as char {
                    '\\' => !escaped,
                    _ => {
                        s.push(c);
                        false
                    }
                };
                *index += 1;
                c = input[*index];
            }
            *index += 1;
            return Ok(Some(Lex::Brace(
                Brace::Quote(s.into_iter().map(|x| x as char).collect()),
                BraceStatus::Agnostic,
                0,
                *index,
            )));
        }
        BraceContinuation::Char => {
            return match input[*index] as char {
                '\'' => Err(LexParseError::NoCharBetweenSingleQuotes(*index)),
                '\\' => {
                    *index += 1;
                    let c = input[*index];
                    *index += 1;
                    if input[*index] != 27 {
                        Err(LexParseError::MultipleCharsBetweenSingleQuotes(*index))
                    } else {
                        *index += 1;
                        Ok(Some(Lex::Brace(
                            Brace::Char(c),
                            BraceStatus::Agnostic,
                            0,
                            *index,
                        )))
                    }
                }
                _ => {
                    let c = input[*index];
                    *index += 1;
                    if input[*index] != 27 {
                        Err(LexParseError::NoCharBetweenSingleQuotes(*index))
                    } else {
                        *index += 1;
                        Ok(Some(Lex::Brace(
                            Brace::Char(c),
                            BraceStatus::Agnostic,
                            0,
                            *index,
                        )))
                    }
                }
            };
        }
        BraceContinuation::Comments => {
            let mut c = input[*index];
            let mut s = Vec::new();
            match c as char {
                '/' => {
                    while *index < input.len() && c != 10 {
                        s.push(c);
                        *index += 1;
                        c = input[*index];
                    }
                }
                '*' => {
                    while *index < input.len() && c != 10 {
                        s.push(c);
                        *index += 1;
                        c = input[*index];
                    }
                }
                _ => {
                    return Err(LexParseError::InvalidCharacter(*index));
                }
            };
            *index += 1;
            return Ok(Some(Lex::Brace(
                Brace::Comment(s.into_iter().map(|x| x as char).collect()),
                BraceStatus::Agnostic,
                0,
                *index,
            )));
        }
    }
}
pub fn take_brace(c: u8, _context: &mut BraceContext) -> Option<BraceContinuation> {
    match c as char {
        '(' => Some(BraceContinuation::Paranthese(BraceStatus::SpeculativeOpen)),
        ')' => Some(BraceContinuation::Paranthese(BraceStatus::Close)),
        '[' => Some(BraceContinuation::SquareBracket(
            BraceStatus::SpeculativeOpen,
        )),
        ']' => Some(BraceContinuation::SquareBracket(BraceStatus::Close)),
        '{' => Some(BraceContinuation::CurlyBrackets(
            BraceStatus::SpeculativeOpen,
        )),
        '}' => Some(BraceContinuation::CurlyBrackets(BraceStatus::Close)),
        '<' => Some(BraceContinuation::AngleBrackets(
            BraceStatus::SpeculativeOpen,
        )),
        '>' => Some(BraceContinuation::AngleBrackets(BraceStatus::Close)),
        '\"' => Some(BraceContinuation::Quote),
        '\'' => Some(BraceContinuation::Char),
        '/' => Some(BraceContinuation::Comments),
        _ => None,
    }
}

pub enum BraceContinuation {
    Paranthese(BraceStatus),
    SquareBracket(BraceStatus),
    CurlyBrackets(BraceStatus),
    AngleBrackets(BraceStatus),
    Quote,
    Char,
    Comments,
}
