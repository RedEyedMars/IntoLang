use crate::parse::constant::{Brace, Delimiter, Keyword, Operator};
use std::collections::HashMap;
mod brace;
mod delimiter;
mod identifier;
mod number;
mod operator;

use crate::parse::lex::brace::{lex_brace, take_brace};
use crate::parse::lex::delimiter::{is_delimiter, lex_delim};
use crate::parse::lex::identifier::{is_ident_start, lex_ident};
use crate::parse::lex::number::{is_number_start, lex_num};
use crate::parse::lex::operator::{lex_op, start_operator};

#[derive(PartialEq, Eq, Debug)]
pub enum Lex {
    Identifier(String, usize),
    Keyword(Keyword, usize),
    Integer(String, usize),
    Float(String, usize),
    Operator(Operator, usize),
    Brace(Brace, BraceStatus, usize, usize),
    Delimiter(Delimiter),
}
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BraceStatus {
    SpeculativeOpen,
    Open(usize),
    Close,
    Agnostic,
}
#[derive(PartialEq, Eq, Debug)]
pub struct BraceContext {
    paranthese_map: HashMap<usize, usize>,
    squares_map: HashMap<usize, usize>,
    curls_map: HashMap<usize, usize>,
    angles_map: HashMap<usize, usize>,
    //
    paranthese: Vec<usize>,
    squares: Vec<usize>,
    curls: Vec<usize>,
    angles: Vec<usize>,
}
impl BraceContext {
    pub fn new() -> BraceContext {
        return BraceContext {
            paranthese_map: HashMap::new(),
            squares_map: HashMap::new(),
            curls_map: HashMap::new(),
            angles_map: HashMap::new(),
            //
            paranthese: Vec::new(),
            squares: Vec::new(),
            curls: Vec::new(),
            angles: Vec::new(),
        };
    }
    pub fn distribute_braces(&self, lexes: &mut Vec<Lex>) {
        for (insert_at, len_of_scope) in self.paranthese_map.iter() {
            let (level, pos) = match lexes[*insert_at] {
                Lex::Brace(_, _, level, pos) => (level, pos),
                _ => (0usize, 0usize),
            };
            std::mem::replace(
                &mut lexes[*insert_at],
                Lex::Brace(Brace::Brace, BraceStatus::Open(*len_of_scope), level, pos),
            );
        }
        for (insert_at, len_of_scope) in self.squares_map.iter() {
            let (level, pos) = match lexes[*insert_at] {
                Lex::Brace(_, _, level, pos) => (level, pos),
                _ => (0usize, 0usize),
            };
            std::mem::replace(
                &mut lexes[*insert_at],
                Lex::Brace(Brace::Square, BraceStatus::Open(*len_of_scope), level, pos),
            );
        }
        for (insert_at, len_of_scope) in self.curls_map.iter() {
            let (level, pos) = match lexes[*insert_at] {
                Lex::Brace(_, _, level, pos) => (level, pos),
                _ => (0usize, 0usize),
            };
            std::mem::replace(
                &mut lexes[*insert_at],
                Lex::Brace(Brace::Bracket, BraceStatus::Open(*len_of_scope), level, pos),
            );
        }
        for (insert_at, len_of_scope) in self.angles_map.iter() {
            let (level, pos) = match lexes[*insert_at] {
                Lex::Brace(_, _, level, pos) => (level, pos),
                _ => (0usize, 0usize),
            };
            std::mem::replace(
                &mut lexes[*insert_at],
                Lex::Brace(Brace::Angle, BraceStatus::Open(*len_of_scope), level, pos),
            );
        }
    }

    pub fn update_refs(
        &mut self,
        brace: Brace,
        status: BraceStatus,
        index: usize,
        lex_index: usize,
    ) -> Result<usize, LexParseError> {
        let stack = match brace {
            Brace::Brace => &mut self.paranthese,
            Brace::Angle => &mut self.angles,
            Brace::Bracket => &mut self.curls,
            Brace::Square => &mut self.squares,
            _ => &mut self.paranthese,
        };
        match status {
            BraceStatus::SpeculativeOpen => {
                stack.push(lex_index);
                Ok(stack.len() - 1usize)
            }
            BraceStatus::Close => {
                if let Some(index_to_insert) = stack.pop() {
                    match brace {
                        Brace::Brace => &mut self.paranthese_map,
                        Brace::Angle => &mut self.angles_map,
                        Brace::Bracket => &mut self.curls_map,
                        Brace::Square => &mut self.squares_map,
                        _ => &mut self.paranthese_map,
                    }
                    .insert(index_to_insert, lex_index);
                    Ok(stack.len() + 1usize)
                } else {
                    Err(LexParseError::InvalidDelimiter(index))
                }
            }
            BraceStatus::Agnostic => Ok(0usize),
            BraceStatus::Open(_) => Err(LexParseError::InvalidDelimiter(index)),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum LexParseError {
    InvalidDelimiter(usize),
    QuoteNotEnded(usize),
    NoCharBetweenSingleQuotes(usize),
    MultipleCharsBetweenSingleQuotes(usize),
    InvalidCharacter(usize),
}
pub fn lex(
    input: &[u8],
    index: &mut usize,
    lex_index: &usize,
    brace_context: &mut BraceContext,
) -> Result<Option<Lex>, LexParseError> {
    if *index >= input.len() {
        return Ok(None);
    } else {
        let mut c: u8 = input[*index];
        while is_whitespace(c) {
            *index += 1;
            if *index >= input.len() {
                return Ok(None);
            }
            c = input[*index];
        }
        let fallback = *index;
        if is_ident_start(c) {
            return lex_ident(input, index, c);
        } else if is_delimiter(c) {
            *index = *index + 1;
            return lex_delim(c);
        } else if is_number_start(c) {
            return lex_num(input, index, c);
        } else if let Some(op) = start_operator(c, *index) {
            return lex_op(input, index, op);
        } else if let Some(brace) = take_brace(c, brace_context) {
            return lex_brace(input, index, brace, lex_index, brace_context);
        }

        *index = fallback;
        Ok(None)
    }
}

fn is_whitespace(c: u8) -> bool {
    match c as char {
        ' ' => true,
        '\t' => true,
        '\n' => true,
        '\r' => true,
        _ => false,
    }
}
