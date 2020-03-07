use crate::parse::constant::{Brace, Keyword, Number, Operator};
use crate::parse::context::{BraceState, ContextScope, TokenizerContext};
use crate::parse::lex::{lex, BraceContext, BraceStatus, Lex, LexParseError};

use std::cell::RefCell;

#[derive(Debug, PartialEq, Eq)]
pub enum Literal {
    Keyword(Keyword),
    Identifier(String),
    VariableDeclaration(String, String),
    Number(Number),
    String(String),
    Comment(String),
    Void,
}
#[derive(Debug, PartialEq, Eq)]
pub enum OperatorGroup {
    UniOperator(Operator, Box<Token>),
    BiOperator(Operator, Box<Token>, Box<Token>),
    TriOperator(Operator, Box<Token>, Box<Token>, Box<Token>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Literal(Literal),
    Operator(OperatorGroup),
    Block(Brace, usize),
    AgrandizedString(String, Box<Token>, String),
    TypeDef(Literal, Literal, Box<Token>), //Classifier, Identifier, Body
    MethodDef(Keyword, String, Box<Token>, Box<Token>),
    Delimiter,
}
#[derive(Debug, PartialEq, Eq)]
pub enum TokenParseError {
    Lex(LexParseError),
    UnableToFindNextLex(usize),
    ContextTriedToEscapeRootScope,
    UnsupportedTokenizerOperator,
    UnsupportedKeywordToken(Keyword),
    FirstArgumentIsNotAnIdentifier,
    Stalled,
    ExpectedOperandButFoundNone,
    OperatorIsNotUniary(Operator, usize),
    ParsedCloseBraceAtIncorrectScopeLevel(usize, usize),
    ExpectedBraceButFoundOtherBrace(Brace, Brace),
    AttemptedToParseCloseBraceWithoutOpen(Brace),
    AttemptedToRetrieveScopeFromABraceStateOfNone,
    ExpectedIdentifierNameAfterTypeDef(usize),
    ExpectedTypeClassifierAfterTypeDefIdentifier(usize),
    ExpectedTypeBodyAfterTypeDef(usize),
    TypeRequiresNameAndClassifier,
    ExpectedBodyAfterMethodSignature,
    ExpectedParametersAfterMethodName,
    ExpectedMethodName,
}

pub fn parse_lexs(input: &[u8]) -> Result<Vec<Lex>, TokenParseError> {
    let mut result = Vec::with_capacity(input.len());
    let mut index = 0usize;
    let mut brace_context = BraceContext::new();
    while index < input.len() {
        let previous = index;
        if let Some(token) = match lex(input, &mut index, &result.len(), &mut brace_context) {
            Ok(lex) => lex,
            Err(err) => return Err(TokenParseError::Lex(err)),
        } {
            result.push(token);
        }
        if previous == index {
            return Err(TokenParseError::UnableToFindNextLex(index));
        }
    }
    //TODO verify braces are all closed
    brace_context.distribute_braces(&mut result);
    Ok(result)
}

pub fn parse_tokens(input: &[u8]) -> Result<TokenizerContext, TokenParseError> {
    let mut context = TokenizerContext::new();
    let lexes = parse_lexs(input)?;
    let mut index = 0usize;
    let mut previous_index = std::usize::MAX;

    parse_tokens_from_lexes(
        &mut index,
        &mut previous_index,
        lexes.len(),
        &lexes,
        &mut context,
    )?;
    Ok(context)
}

fn parse_tokens_from_lexes<'a>(
    index: &mut usize,
    previous_index: &mut usize,
    length: usize,
    lexes: &Vec<Lex>,
    context: &'a mut TokenizerContext,
) -> Result<(), TokenParseError> {
    while *index < length {
        next_token(&lexes, index, length, context)?;
        if *index == *previous_index {
            return Err(TokenParseError::Stalled);
        } else {
            *previous_index = *index;
        }
    }
    Ok(())
}

fn next_token<'a>(
    lexes: &Vec<Lex>,
    index: &mut usize,
    length: usize,
    context: &'a mut TokenizerContext,
) -> Result<(), TokenParseError> {
    while match lexes.get(*index) {
        Some(Lex::Delimiter(_)) => true,
        _ => false,
    } {
        *index += 1;
    }
    if *index >= length {
        return Ok(());
    }
    match match lexes.get(*index).unwrap() {
        //Literals
        Lex::Identifier(name, _) => {
            context.push_identifier(name.clone());
            if *index + 1 < lexes.len() {
                *index = *index + 1;
                match lexes.get(*index).unwrap() {
                    Lex::Identifier(variable_name, _) => Ok(Literal::as_type_declaration(
                        name.clone(),
                        variable_name.clone(),
                    )),
                    _ => {
                        *index = *index - 1;
                        Ok(Literal::as_identifier(name.clone()))
                    }
                }
            } else {
                Ok(Literal::as_identifier(name.clone()))
            }
        }
        Lex::Keyword(Keyword::Calc, _) => {
            push_method_declaration(Keyword::Calc, index, lexes, context)
        }
        Lex::Keyword(Keyword::Trans, _) => {
            push_method_declaration(Keyword::Trans, index, lexes, context)
        }
        Lex::Keyword(Keyword::Type, pos) => {
            if *index + 4 >= lexes.len() {
                return Err(TokenParseError::TypeRequiresNameAndClassifier);
            }
            *index = *index + 1;
            if let Some(Lex::Identifier(name, name_pos)) = lexes.get(*index) {
                *index = *index + 1;
                if let Some(Lex::Keyword(classifier, _)) = lexes.get(*index) {
                    *index = *index + 1;
                    next_token(lexes, index, length, context)?;
                    if let Some(body_box) = context.pop_token() {
                        Ok(Box::new(Token::TypeDef(
                            Literal::Keyword(*classifier),
                            Literal::Identifier(name.clone()),
                            match *body_box {
                                Token::Block(_, _) => Ok(body_box),
                                _ => Err(TokenParseError::ExpectedTypeBodyAfterTypeDef(*pos)),
                            }?,
                        )))
                    } else {
                        Err(TokenParseError::ExpectedTypeBodyAfterTypeDef(*pos))
                    }
                } else {
                    Err(TokenParseError::ExpectedTypeClassifierAfterTypeDefIdentifier(*pos))
                }
            } else {
                Err(TokenParseError::ExpectedIdentifierNameAfterTypeDef(*pos))
            }
        }
        Lex::Keyword(key, _) => Ok(Literal::as_keyword(key.clone())),
        Lex::Integer(i, _) => Ok(Literal::as_integer(i.parse::<i64>().unwrap())),
        Lex::Float(f, _) => Ok(Literal::as_float(f.parse::<f64>().unwrap())),
        Lex::Brace(Brace::Quote(s), _, _, _) => Ok(Literal::as_string(s.clone())),
        Lex::Brace(Brace::Comment(c), _, _, _) => Ok(Literal::as_comment(c.clone())),
        //Braces
        Lex::Brace(brace, BraceStatus::Open(len), level, _) => {
            if let Brace::Brace = brace {
                if let Some(Lex::Brace(Brace::Brace, BraceStatus::Close, _, _)) =
                    lexes.get(*index + 1)
                {
                    *index += 1;
                    Ok(Box::new(Token::Literal(Literal::Void)))
                } else {
                    push_braced_block(brace, *len, *level, index, lexes, context)
                }
            } else {
                push_braced_block(brace, *len, *level, index, lexes, context)
            }
        }
        //Operators
        Lex::Operator(op, pos) => {
            if let Some(previous_token) = context.pop_token() {
                Ok(OperatorGroup::as_bi_op(
                    previous_token,
                    *op,
                    *pos,
                    lexes,
                    index,
                    context,
                )?)
            } else {
                Ok(OperatorGroup::as_uni_op(*op, *pos, lexes, index, context)?)
            }
        }
        _ => Err(TokenParseError::UnsupportedTokenizerOperator),
    } {
        Ok(result) => {
            context.push_token(result);
            *index = *index + 1;
            Ok(())
        }
        Err(err) => Err(err),
    }
}

fn push_method_declaration(
    mode: Keyword,
    index: &mut usize,
    lexes: &Vec<Lex>,
    context: &mut TokenizerContext,
) -> Result<Box<Token>, TokenParseError> {
    *index += 1;
    if let Some(Lex::Identifier(name, _)) = lexes.get(*index) {
        *index += 1;
        if let Some(Lex::Brace(Brace::Brace, BraceStatus::Open(len), level, _)) = lexes.get(*index)
        {
            let parameters = push_braced_block(&Brace::Brace, *len, *level, index, lexes, context)?;
            *index += 1;
            if let Some(Lex::Brace(Brace::Bracket, BraceStatus::Open(len), level, _)) =
                lexes.get(*index)
            {
                let body = push_braced_block(&Brace::Bracket, *len, *level, index, lexes, context)?;
                Ok(Box::new(Token::MethodDef(
                    mode,
                    name.clone(),
                    parameters,
                    body,
                )))
            } else {
                Err(TokenParseError::ExpectedBodyAfterMethodSignature)
            }
        } else {
            Err(TokenParseError::ExpectedParametersAfterMethodName)
        }
    } else {
        Err(TokenParseError::ExpectedMethodName)
    }
}

fn push_braced_block(
    brace: &Brace,
    len: usize,
    level: usize,
    index: &mut usize,
    lexes: &Vec<Lex>,
    context: &mut TokenizerContext,
) -> Result<Box<Token>, TokenParseError> {
    context.push_scope(
        context.get_state(),
        BraceState::Braced(brace.clone(), level),
    );
    *index = *index + 1;
    parse_tokens_from_lexes(index, &mut (*index - 1), len, lexes, context)?;
    let scope_index = context.current_scope().get_index();
    context.pop_scope()?;
    *index = len;
    Ok(Box::new(Token::Block(brace.clone(), scope_index)))
}

impl Literal {
    fn as_identifier(ident: String) -> Box<Token> {
        Box::new(Token::Literal(Literal::Identifier(ident)))
    }
    fn as_keyword(keyword: Keyword) -> Box<Token> {
        Box::new(Token::Literal(Literal::Keyword(keyword)))
    }
    fn as_type_declaration(type_name: String, name: String) -> Box<Token> {
        Box::new(Token::Literal(Literal::VariableDeclaration(
            type_name, name,
        )))
    }

    fn as_integer(i: i64) -> Box<Token> {
        Box::new(Token::Literal(Literal::Number(Number::Integer(i))))
    }
    fn as_float(f: f64) -> Box<Token> {
        Box::new(Token::Literal(Literal::Number(Number::Float(f))))
    }
    fn as_string(s: String) -> Box<Token> {
        Box::new(Token::Literal(Literal::String(s)))
    }
    fn as_comment(c: String) -> Box<Token> {
        Box::new(Token::Literal(Literal::Comment(c)))
    }
}

impl OperatorGroup {
    fn as_uni(operand: Box<Token>, op: Operator) -> Box<Token> {
        Box::new(Token::Operator(OperatorGroup::UniOperator(op, operand)))
    }
    fn as_bi(operand: Box<Token>, op: Operator, parameter: Box<Token>) -> Box<Token> {
        Box::new(Token::Operator(OperatorGroup::BiOperator(
            op, operand, parameter,
        )))
    }

    fn as_tri(
        operand: Box<Token>,
        op: Operator,
        parameter1: Box<Token>,
        parameter2: Box<Token>,
    ) -> Box<Token> {
        Box::new(Token::Operator(OperatorGroup::TriOperator(
            op, operand, parameter1, parameter2,
        )))
    }

    fn as_uni_op(
        op: Operator,
        pos: usize,
        lexes: &Vec<Lex>,
        index: &mut usize,
        context: &mut TokenizerContext,
    ) -> Result<Box<Token>, TokenParseError> {
        if let Err(e) = match op {
            Operator::Not => Ok(()),
            _ => Err(TokenParseError::OperatorIsNotUniary(op, pos)),
        } {
            return Err(e);
        }
        *index = *index + 1;
        next_token(lexes, index, std::usize::MAX, context)?;
        match context.pop_token() {
            Some(token) => Ok(OperatorGroup::as_uni(token, op)),
            None => Err(TokenParseError::ExpectedOperandButFoundNone),
        }
    }

    fn as_bi_op(
        previous_token: Box<Token>,
        op: Operator,
        pos: usize,
        lexes: &Vec<Lex>,
        index: &mut usize,
        context: &mut TokenizerContext,
    ) -> Result<Box<Token>, TokenParseError> {
        *index = *index + 1;
        next_token(lexes, index, std::usize::MAX, context)?;
        match context.pop_token() {
            Some(token) => Ok(OperatorGroup::as_bi(previous_token, op, token)),
            None => Err(TokenParseError::ExpectedOperandButFoundNone),
        }
    }
}

//--------===============---------
//--------=====TESTS=====---------
//--------===============---------

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_type_tokens() -> Result<(), TokenParseError> {
        let context = parse_tokens(b"type Geheusie data { int x, int y, }")?;
        let expected_scope_index = 1;
        context
            .current_scope()
            .assert_eq(vec![Box::new(Token::TypeDef(
                Literal::Keyword(Keyword::Data),
                Literal::Identifier("Geheusie".to_string()),
                Box::new(Token::Block(Brace::Bracket, expected_scope_index)),
            ))]);
        context
            .get_scope(expected_scope_index)
            .unwrap()
            .assert_eq(vec![
                Box::new(Token::Literal(Literal::VariableDeclaration(
                    "int".to_string(),
                    "x".to_string(),
                ))),
                Box::new(Token::Literal(Literal::VariableDeclaration(
                    "int".to_string(),
                    "y".to_string(),
                ))),
            ]);
        Ok(())
    }
    #[test]
    fn test_parse_basic_braces() -> Result<(), TokenParseError> {
        let context = parse_tokens(b"(Goose)")?;
        let expected_scope_index = 1;
        let expected_outer = Box::new(Token::Block(Brace::Brace, expected_scope_index));
        context.current_scope().assert_eq(vec![expected_outer]);
        context
            .get_scope(expected_scope_index)
            .unwrap()
            .assert_eq(vec![Literal::as_identifier("Goose".to_string())]);
        Ok(())
    }

    #[test]
    fn test_parse_uni_op_token() -> Result<(), TokenParseError> {
        parse_tokens(b"!Puff")?
            .current_scope()
            .assert_eq(vec![OperatorGroup::as_uni(
                Literal::as_identifier("Puff".to_string()),
                Operator::Not,
            )]);
        Ok(())
    }
    #[test]
    fn test_parse_bi_op_token() -> Result<(), TokenParseError> {
        parse_tokens(b"Goose + Ocelot")?
            .current_scope()
            .assert_eq(vec![OperatorGroup::as_bi(
                Literal::as_identifier("Goose".to_string()),
                Operator::Plus,
                Literal::as_identifier("Ocelot".to_string()),
            )]);
        Ok(())
    }
    #[test]
    fn test_parse_identifier_token() -> Result<(), TokenParseError> {
        parse_tokens(b"Goose")?
            .current_scope()
            .assert_eq(vec![Box::new(Token::Literal(Literal::Identifier(
                "Goose".to_string(),
            )))]);
        parse_tokens(b"()")?
            .current_scope()
            .assert_eq(vec![Box::new(Token::Literal(Literal::Void))]);
        Ok(())
    }

    #[test]
    fn test_lex_identifiers() -> Result<(), TokenParseError> {
        assert_eq!(
            parse_lexs(b"Goose")?,
            vec!(Lex::Identifier("Goose".to_string(), 5))
        );
        assert_eq!(parse_lexs(b"G")?, vec!(Lex::Identifier("G".to_string(), 1)));
        assert_eq!(parse_lexs(b"data")?, vec!(Lex::Keyword(Keyword::Data, 4)));
        assert_eq!(parse_lexs(b"comp")?, vec!(Lex::Keyword(Keyword::Comp, 4)));
        assert_eq!(parse_lexs(b"calc")?, vec!(Lex::Keyword(Keyword::Calc, 4)));
        Ok(())
    }
    #[test]
    fn test_lex_numbers() -> Result<(), TokenParseError> {
        assert_eq!(parse_lexs(b"1")?, vec!(Lex::Integer("1".to_string(), 1)));
        assert_eq!(parse_lexs(b"10")?, vec!(Lex::Integer("10".to_string(), 2)));
        assert_eq!(parse_lexs(b"1.0")?, vec!(Lex::Float("1.0".to_string(), 3)));
        Ok(())
    }
    #[test]
    fn test_lex_operators() -> Result<(), TokenParseError> {
        assert_eq!(
            parse_lexs(b".")?,
            vec!(Lex::Operator(Operator::Accessor, 1))
        );
        assert_eq!(
            parse_lexs(b"..")?,
            vec!(Lex::Operator(Operator::RangeMiddle, 2))
        );
        assert_eq!(
            parse_lexs(b"...")?,
            vec!(Lex::Operator(Operator::ArrayContinuation, 3))
        );

        assert_eq!(parse_lexs(b"+")?, vec!(Lex::Operator(Operator::Plus, 1)));
        assert_eq!(
            parse_lexs(b"+=")?,
            vec!(Lex::Operator(Operator::PlusEquals, 2))
        );
        Ok(())
    }
    #[test]
    fn test_lex_braces() -> Result<(), TokenParseError> {
        assert_eq!(
            parse_lexs(b"(")?,
            vec!(Lex::Brace(Brace::Brace, BraceStatus::SpeculativeOpen, 0, 1))
        );
        assert_eq!(
            parse_lexs(b"{}")?,
            vec!(
                Lex::Brace(Brace::Bracket, BraceStatus::Open(1usize), 0, 1),
                Lex::Brace(Brace::Bracket, BraceStatus::Close, 1, 2)
            )
        );
        assert_eq!(
            parse_lexs(b"{}}"),
            Err(TokenParseError::Lex(LexParseError::InvalidDelimiter(3)))
        );

        assert_eq!(
            parse_lexs(b"{data Goose}")?,
            vec!(
                Lex::Brace(Brace::Bracket, BraceStatus::Open(3usize), 0, 1),
                Lex::Keyword(Keyword::Data, 5),
                Lex::Identifier("Goose".to_string(), 11),
                Lex::Brace(Brace::Bracket, BraceStatus::Close, 1, 12)
            )
        );
        Ok(())
    }
}
