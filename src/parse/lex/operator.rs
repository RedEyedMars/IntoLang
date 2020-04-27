use crate::parse::constant::Operator;
use crate::parse::lex::{Lex, LexParseError};

pub fn lex_op(
    input: &[u8],
    index: &mut usize,
    mut acc: OperatorContinuation,
) -> Result<Option<Lex>, LexParseError> {
    *index += 1;
    while *index < input.len() {
        if let Some(next) = acc.next(input[*index], *index) {
            acc = next;
            *index += 1;
        } else {
            break;
        }
    }
    return Ok(acc.as_lex());
}

pub fn start_operator(c: u8, index: usize) -> Option<OperatorContinuation> {
    match c as char {
        '+' => Some(OperatorContinuation::Plus(index)),
        '-' => Some(OperatorContinuation::Dash(index)),
        '/' => Some(OperatorContinuation::ForwardSlash(index)),
        '=' => Some(OperatorContinuation::Equals(index)),
        '&' => Some(OperatorContinuation::Ampersand(index)),
        '!' => Some(OperatorContinuation::Not(index)),
        '@' => None,
        '#' => None,
        '$' => None,
        '%' => Some(OperatorContinuation::Percent(index)),
        '^' => Some(OperatorContinuation::Caret(index)),
        '*' => Some(OperatorContinuation::Asterisk(index)),
        '<' => Some(OperatorContinuation::LessThan(index)),
        '>' => Some(OperatorContinuation::GreaterThan(index)),
        '|' => Some(OperatorContinuation::Pipe(index)),
        '.' => Some(OperatorContinuation::Dot(index)),
        '?' => Some(OperatorContinuation::QuestionMark(index)),
        ':' => Some(OperatorContinuation::Colon(index)),
        _ => None,
    }
}
pub enum OperatorContinuation {
    Plus(usize),
    PlusEquals(usize),
    Dash(usize),
    MinusEquals(usize),
    Arrow(usize),
    Equals(usize),
    DoubleEquals(usize),
    Ampersand(usize),
    Pipe(usize),
    Asterisk(usize),
    AsteriskEquals(usize),
    Dot(usize),
    DoubleDot(usize),
    TripleDot(usize),
    Not(usize),
    NotEquals(usize),
    QuestionMark(usize),
    Colon(usize),
    DoubleColon(usize),
    ForwardSlash(usize),
    ForwardSlashEquals(usize),
    //    BackSlash(usize),
    Percent(usize),
    Caret(usize),
    LessThan(usize),
    LessThanEquals(usize),
    GreaterThan(usize),
    GreaterThanEquals(usize),
    Into(usize),
}
impl OperatorContinuation {
    fn as_lex(&self) -> Option<Lex> {
        match *self {
            OperatorContinuation::Plus(index) => Some(Lex::Operator(Operator::Plus, index + 1)),
            OperatorContinuation::PlusEquals(index) => {
                Some(Lex::Operator(Operator::PlusEquals, index + 1))
            }
            OperatorContinuation::Dash(index) => Some(Lex::Operator(Operator::Minus, index + 1)),
            OperatorContinuation::MinusEquals(index) => {
                Some(Lex::Operator(Operator::MinusEquals, index + 1))
            }
            OperatorContinuation::Asterisk(index) => {
                Some(Lex::Operator(Operator::Multiply, index + 1))
            }
            OperatorContinuation::AsteriskEquals(index) => {
                Some(Lex::Operator(Operator::MultiplyEquals, index + 1))
            }
            OperatorContinuation::ForwardSlash(index) => {
                Some(Lex::Operator(Operator::Divide, index + 1))
            }
            OperatorContinuation::ForwardSlashEquals(index) => {
                Some(Lex::Operator(Operator::DivideEquals, index + 1))
            }
            OperatorContinuation::Caret(index) => Some(Lex::Operator(Operator::PowerOf, index + 1)),
            OperatorContinuation::Percent(index) => {
                Some(Lex::Operator(Operator::Modulus, index + 1))
            }
            OperatorContinuation::Equals(index) => {
                Some(Lex::Operator(Operator::Assignment, index + 1))
            }
            OperatorContinuation::Not(index) => Some(Lex::Operator(Operator::Not, index + 1)),
            OperatorContinuation::DoubleEquals(index) => {
                Some(Lex::Operator(Operator::IsEquals, index + 1))
            }
            OperatorContinuation::NotEquals(index) => {
                Some(Lex::Operator(Operator::IsNotEquals, index + 1))
            }
            OperatorContinuation::LessThan(index) => Some(Lex::Operator(Operator::LessThan, index)),
            OperatorContinuation::LessThanEquals(index) => {
                Some(Lex::Operator(Operator::LessThanOrEquals, index + 1))
            }
            OperatorContinuation::GreaterThanEquals(index) => {
                Some(Lex::Operator(Operator::GreaterThanOrEquals, index + 1))
            }
            OperatorContinuation::GreaterThan(index) => {
                Some(Lex::Operator(Operator::GreaterThan, index + 1))
            }
            OperatorContinuation::Ampersand(index) => Some(Lex::Operator(Operator::And, index + 1)),
            OperatorContinuation::Pipe(index) => Some(Lex::Operator(Operator::Or, index + 1)),
            OperatorContinuation::Dot(index) => Some(Lex::Operator(Operator::Accessor, index + 1)),
            OperatorContinuation::Arrow(index) => Some(Lex::Operator(Operator::Arrow, index + 1)),
            OperatorContinuation::DoubleDot(index) => {
                Some(Lex::Operator(Operator::RangeMiddle, index + 1))
            }
            OperatorContinuation::TripleDot(index) => {
                Some(Lex::Operator(Operator::ArrayContinuation, index + 1))
            }
            OperatorContinuation::Colon(index) => Some(Lex::Operator(Operator::Of, index + 1)),
            OperatorContinuation::DoubleColon(index) => {
                Some(Lex::Operator(Operator::OfClass, index + 1))
            }
            //OperatorContinuation::BackSlash(index) => {
            //    Some(Lex::Operator(Operator::Escape, index + 1))
            //}
            OperatorContinuation::QuestionMark(index) => {
                Some(Lex::Operator(Operator::QuestionMark, index + 1))
            }
            OperatorContinuation::Into(index) => Some(Lex::Operator(Operator::Into, index + 1)),
        }
    }

    fn next(&self, c: u8, index: usize) -> Option<OperatorContinuation> {
        match *self {
            OperatorContinuation::Plus(_) => match c as char {
                '=' => Some(OperatorContinuation::PlusEquals(index)),
                _ => None,
            },
            OperatorContinuation::PlusEquals(_) => None,
            OperatorContinuation::Dash(_) => match c as char {
                '=' => Some(OperatorContinuation::MinusEquals(index)),
                '>' => Some(OperatorContinuation::Arrow(index)),
                _ => None,
            },
            OperatorContinuation::MinusEquals(_) => None,
            OperatorContinuation::Asterisk(_) => match c as char {
                '=' => Some(OperatorContinuation::AsteriskEquals(index)),
                _ => None,
            },
            OperatorContinuation::AsteriskEquals(_) => None,
            OperatorContinuation::ForwardSlash(_) => match c as char {
                '=' => Some(OperatorContinuation::ForwardSlashEquals(index)),
                _ => None,
            },
            OperatorContinuation::ForwardSlashEquals(_) => None,
            OperatorContinuation::Caret(_) => None,
            OperatorContinuation::Percent(_) => None,
            OperatorContinuation::Equals(_) => match c as char {
                '=' => Some(OperatorContinuation::DoubleEquals(index)),
                '>' => Some(OperatorContinuation::Into(index)),
                _ => None,
            },
            OperatorContinuation::DoubleEquals(_) => None,
            OperatorContinuation::Not(_) => match c as char {
                '=' => Some(OperatorContinuation::NotEquals(index)),
                _ => None,
            },
            OperatorContinuation::NotEquals(_) => None,
            OperatorContinuation::LessThan(_) => match c as char {
                '=' => Some(OperatorContinuation::LessThanEquals(index)),
                _ => None,
            },
            OperatorContinuation::LessThanEquals(_) => None,
            OperatorContinuation::GreaterThan(_) => match c as char {
                '=' => Some(OperatorContinuation::GreaterThanEquals(index)),
                _ => None,
            },
            OperatorContinuation::GreaterThanEquals(_) => None,
            OperatorContinuation::Ampersand(_) => None,
            OperatorContinuation::Pipe(_) => None,
            OperatorContinuation::Arrow(_) => None,
            OperatorContinuation::Into(_) => None,
            OperatorContinuation::Dot(_) => match c as char {
                '.' => Some(OperatorContinuation::DoubleDot(index)),
                _ => None,
            },
            OperatorContinuation::DoubleDot(_) => match c as char {
                '.' => Some(OperatorContinuation::TripleDot(index)),
                _ => None,
            },
            OperatorContinuation::TripleDot(_) => None,
            OperatorContinuation::Colon(_) => match c as char {
                ':' => Some(OperatorContinuation::DoubleColon(index)),
                _ => None,
            },
            OperatorContinuation::DoubleColon(_) => None,
            //OperatorContinuation::BackSlash(_) => None,
            OperatorContinuation::QuestionMark(_) => None,
        }
    }
}
