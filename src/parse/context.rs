use crate::parse::constant::Brace;
use crate::parse::token::{Token, TokenParseError};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug)]
pub enum ContextState {
    Root,
    Global,
}

#[derive(Debug, Clone)]
pub enum BraceState {
    None,
    Braced(Brace, usize),
}
#[derive(Debug)]
pub struct ContextScope {
    state: ContextState,
    brace_state: BraceState,
    undeclared_identifiers: HashSet<String>,
    declared_identifiers: HashSet<String>,
    tokens: Vec<Box<Token>>,
    index: usize,
    parent: usize,
}
impl ContextScope {
    pub fn new(index: usize) -> ContextScope {
        ContextScope {
            state: ContextState::Root,
            brace_state: BraceState::None,
            undeclared_identifiers: HashSet::new(),
            declared_identifiers: HashSet::new(),
            tokens: Vec::new(),
            index: index,
            parent: std::usize::MAX,
        }
    }
    pub fn with_parent(
        new_state: ContextState,
        new_brace_state: BraceState,
        index: usize,
        parent: usize,
    ) -> ContextScope {
        ContextScope {
            state: new_state,
            brace_state: new_brace_state,
            undeclared_identifiers: HashSet::new(),
            declared_identifiers: HashSet::new(),
            tokens: Vec::new(),
            index: index,
            parent: parent,
        }
    }
    pub fn get_index(&self) -> usize {
        self.index
    }
    pub fn get_brace_state(&self) -> BraceState {
        self.brace_state.clone()
    }
    pub fn assert_eq(&self, compare_vec: Vec<Box<Token>>) {
        assert_eq!(self.tokens, compare_vec);
    }
    pub fn println(&self) {
        println!("{:?}", self.tokens);
    }
}

pub struct TokenizerContext {
    scope: Vec<ContextScope>,
    current_scope: usize,
}

impl TokenizerContext {
    pub fn new() -> TokenizerContext {
        let mut scope = Vec::new();
        let cs = ContextScope::new(0);
        scope.push(cs);
        TokenizerContext {
            scope: scope,
            current_scope: 0,
        }
    }

    pub fn get_state(&self) -> ContextState {
        self.scope.get(self.current_scope).unwrap().state
    }
    pub fn push_scope(&mut self, new_state: ContextState, new_brace_state: BraceState) {
        let new_scope: ContextScope = ContextScope::with_parent(
            match new_state {
                ContextState::Root => ContextState::Global,
                _ => new_state,
            },
            new_brace_state,
            self.scope.len(),
            self.current_scope,
        );
        self.current_scope = self.scope.len();
        self.scope.push(new_scope);
    }
    pub fn current_scope<'a>(&'a self) -> &'a ContextScope {
        self.scope.get(self.current_scope).unwrap()
    }
    pub fn get_scope<'a>(&'a self, scope_index: usize) -> Option<&'a ContextScope> {
        self.scope.get(scope_index)
    }
    pub fn pop_scope(&mut self) -> Result<(), TokenParseError> {
        match self.scope.get(self.current_scope).unwrap().state {
            ContextState::Root => Err(TokenParseError::ContextTriedToEscapeRootScope),
            _ => {
                self.current_scope = self.scope.get(self.current_scope).unwrap().parent;
                Ok(())
            }
        }
    }

    pub fn push_identifier(&mut self, identifier: String) {
        if !self
            .scope
            .get(self.current_scope)
            .unwrap()
            .declared_identifiers
            .contains(&identifier)
        {
            self.scope
                .get_mut(self.current_scope)
                .unwrap()
                .undeclared_identifiers
                .insert(identifier);
        }
    }

    pub fn push_token(&mut self, token: Box<Token>) {
        self.scope
            .get_mut(self.current_scope)
            .unwrap()
            .tokens
            .push(token);
    }

    pub fn peek_token(&self) -> Option<&Box<Token>> {
        let scope = self.current_scope();
        if scope.tokens.len() > 0 {
            scope.tokens.get(scope.tokens.len() - 1)
        } else {
            None
        }
    }

    pub fn pop_token(&mut self) -> Option<Box<Token>> {
        self.scope.get_mut(self.current_scope).unwrap().tokens.pop()
    }
}
