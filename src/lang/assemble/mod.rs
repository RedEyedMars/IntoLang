use crate::parse::context::TokenizerContext;
use crate::parse::token::{parse_tokens, Literal, Token, TokenParseError};

use std::collections::HashMap;

use std::fs;
use std::io::BufWriter;

use self::context::{AssembledTypeContext, AssemblyFormatContext, ValueFormat, CLASS_ID_VOID};
use self::instruction::AssembledInstruction;

pub mod context;
pub mod instruction;

#[derive(Debug)]
pub enum AssemblyError {
    TypeNotFound,
    TriedToAddVariableToData,
    TokenParseError(TokenParseError),
    NoStartMethodFound,
}

#[derive(Debug, Clone)]
pub struct AssembledData {
    compiled_id: u16,
    index_in_comp: usize,
    memory: u8,
}

impl AssembledData {
    pub fn new(id: u16) -> AssembledData {
        AssembledData {
            compiled_id: id,
            index_in_comp: 0,
            memory: 0,
        }
    }
    pub fn filled(id: u16, index: usize, memory: u8) -> AssembledData {
        AssembledData {
            compiled_id: id,
            index_in_comp: index,
            memory: memory,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssembledComposition {
    compiled_id: u16,
    data: Vec<AssembledObjectAcceptor>,
}
impl AssembledComposition {
    pub fn new(id: u16) -> AssembledComposition {
        AssembledComposition {
            compiled_id: id,
            data: Vec::new(),
        }
    }
    pub fn add_variable(&mut self, v: AssembledObjectAcceptor) {
        self.data.push(v);
    }
}
#[derive(Debug, Clone)]
pub struct AssembledInterface {
    compiled_id: u16,
    impl_id: u16,
}
impl AssembledInterface {
    pub fn new(id: u16, impl_id: u16) -> AssembledInterface {
        AssembledInterface {
            compiled_id: id,
            impl_id: impl_id,
        }
    }
}
#[derive(Debug, Clone)]
pub enum AssembledType {
    Composition(String, AssembledComposition),
    Data(String, AssembledData),
    Interface(String, AssembledInterface),
}

impl AssembledType {
    pub fn get_name(&self) -> String {
        match self {
            AssembledType::Composition(name, _) => name.clone(),
            AssembledType::Data(name, _) => name.clone(),
            AssembledType::Interface(name, _) => name.clone(),
        }
    }
    pub fn get_data<'a>(&'a self, context: &'a AssembledTypeContext) -> Vec<&'a AssembledData> {
        match self {
            AssembledType::Composition(_, comp) => comp
                .data
                .iter()
                .flat_map(|o| context.get_type(&o.class_id).unwrap().get_data(context))
                .collect::<Vec<&AssembledData>>(),
            AssembledType::Data(_, data) => vec![data],
            AssembledType::Interface(_, _) => vec![],
        }
    }
    pub fn get_bytes(&self, context: &AssembledTypeContext) -> u16 {
        match self {
            AssembledType::Composition(_, comp) => comp
                .data
                .iter()
                .map(|d| context.get_type(&d.class_id).unwrap().get_bytes(context))
                .sum(),
            AssembledType::Data(_, data) => data.memory as u16,
            AssembledType::Interface(_, _) => 0u16,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssembledObjectAcceptor {
    source_name: String,
    class_id: u16,
}
impl AssembledObjectAcceptor {
    pub fn new(sn: String, c: u16) -> AssembledObjectAcceptor {
        AssembledObjectAcceptor {
            source_name: sn,
            class_id: c,
        }
    }
}

#[derive(Debug)]
pub struct AssembledMethod {
    name: String,
    consume: ValueFormat,
    produce: ValueFormat,
    produce_type: u16,
    parameters: Vec<AssembledObjectAcceptor>,
    allocation: Vec<AssembledInstruction>,
    body: Vec<AssembledInstruction>,
    free: Vec<AssembledInstruction>,
}
impl AssembledMethod {
    pub fn new(
        name: String,
        consumes: ValueFormat,
        produces: ValueFormat,
        produce_type: u16,
    ) -> AssembledMethod {
        AssembledMethod {
            name: name,
            consume: consumes,
            produce: produces,
            produce_type: produce_type,
            parameters: Vec::new(),
            allocation: Vec::new(),
            body: Vec::new(),
            free: Vec::new(),
        }
    }
    pub fn add_parameter(&mut self, parameter: AssembledObjectAcceptor) {
        self.parameters.push(parameter);
    }
    pub fn add_instruction(&mut self, instruction: AssembledInstruction) {
        self.body.push(instruction);
    }
    pub fn add_allocation(&mut self, allocation: AssembledInstruction) {
        self.allocation.push(allocation);
    }
    pub fn add_free(&mut self, free: AssembledInstruction) {
        self.free.push(free);
    }
    pub fn write(
        &self,
        t: &u16,
        formats: &mut AssemblyFormatContext,
        types: &AssembledTypeContext,
        stream: &mut BufWriter<fs::File>,
    ) -> std::io::Result<()> {
        // convert in into the right format, if neccessary
        if formats.already_in_format(&self.consume) {
            AssembledInstruction::NoOp.write(formats, types, stream)?;
        } else {
            self.consume.convert(t, formats, types, stream)?;
        }
        formats.push(t, self.consume);
        for i in self.body.iter() {
            i.write(formats, types, stream)?;
        }
        formats.pop(&self.produce_type, self.produce);
        Ok(())
    }
}

pub fn assemble(filename: String) -> Result<(), AssemblyError> {
    let mut potential_error = "Could not parse file: ".to_owned();
    let file = filename.to_owned();
    potential_error.push_str(&file);
    let contents = fs::read_to_string(filename).expect(&potential_error);
    match parse_tokens(contents.into_bytes().as_ref()) {
        Ok(context) => {
            context.println();
            assemble_root(context)?;
            Ok(())
        }
        Err(token_error) => Err(AssemblyError::TokenParseError(token_error)),
    }
}

pub fn assemble_root(mut root: TokenizerContext) -> Result<(), AssemblyError> {
    let mut context = AssembledTypeContext::new();
    context.setup_root_scope_types();
    record_declarations(0, &mut root, &mut context)?;
    walk_method(match context
        .get_impl(&CLASS_ID_VOID)
        .unwrap()
        .get(&"start".to_string())
    {
        Some(start_method) => Ok(start_method),
        None => Err(AssemblyError::NoStartMethodFound),
    }?);
    Ok(())
}

fn literal_identifier(identifier: &Literal) -> Result<String, AssemblyError> {
    match identifier.as_identifier_string() {
        Ok(s) => Ok(s),
        Err(e) => Err(AssemblyError::TokenParseError(e)),
    }
}

pub fn record_declarations(
    scope: usize,
    tokens: &mut TokenizerContext,
    type_context: &mut AssembledTypeContext,
) -> Result<(), AssemblyError> {
    let mut types = HashMap::new();
    for t in tokens.get_scope(scope).unwrap().get_tokens().iter() {
        match t.as_ref() {
            Token::TypeDef(identifier, _body) => {
                let type_name = literal_identifier(identifier)?;
                types.insert(type_name.clone(), type_context.create_type(&type_name));
            }
            _ => {}
        }
    }
    type_context.record(types);
    for t in tokens.get_scope(scope).unwrap().get_tokens().iter() {
        match t.as_ref() {
            Token::TypeDef(identifier, body) => {
                if let Literal::Identifier(_) = identifier {
                    if let Token::Block(_, inner_scope) = body.as_ref() {
                        for v in tokens.get_scope(*inner_scope).unwrap().get_tokens().iter() {
                            match v.as_ref() {
                                Token::VariableDef(type_identifier, variable_name) => {
                                    if let Some(var_type) = type_context
                                        .get_type_id(&literal_identifier(type_identifier)?)
                                    {
                                        let variable = AssembledObjectAcceptor::new(
                                            variable_name.clone(),
                                            *var_type,
                                        );
                                        type_context.add_variable(
                                            scope,
                                            &variable_name,
                                            variable,
                                        )?;
                                    } else {
                                        return Err(AssemblyError::TypeNotFound);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn walk_method(_method: &AssembledMethod) {}

  //--------=====================----------\\
 //---------========TESTS========-----------\\
//----------=====================------------\\

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_assemble_declarations() -> Result<(), AssemblyError> {
        assemble("res/test/simple_0.geo".to_string())?;
        Ok(())
    }
}
