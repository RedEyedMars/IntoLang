use crate::lang::assemble::instruction::StreamInstruction;
use crate::lang::assemble::{
    AssembledComposition, AssembledData, AssembledInstruction, AssembledInterface, AssembledMethod,
    AssembledObjectAcceptor, AssembledType, AssemblyError,
};

use std::collections::HashMap;
use std::fs;
use std::io::{BufWriter, Write};

pub const CLASS_ID_VOID: u16 = 0;
pub const CLASS_ID_PRINTABLE: u16 = 1;
pub const CLASS_ID_INT: u16 = 2;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ValueFormat {
    StreamOfElements, // xyzxyzxyz000
    StreamOfValues,   // xxx0yyy0zzz0
                      //Same,             // For Peek operations that do not change the value format
}

impl ValueFormat {
    pub fn convert(
        &self,
        t: &u16,
        formats: &mut AssemblyFormatContext,
        types: &AssembledTypeContext,
        stream: &mut BufWriter<fs::File>,
    ) -> std::io::Result<()> {
        match self {
            //To
            ValueFormat::StreamOfValues => {
                let top = formats.out_format_stack.len();
                //From
                match formats.out_format_stack.get(top).unwrap() {
                    ValueFormat::StreamOfElements => {
                        /*for(int i = 0; out < len; ++i) {
                            in[{variable_index} * {new_t.num_of_bytes} + i] =
                                out[{variable_index} + {old_t.num_of_bytes} * i];
                            out += {old_t.num_of_bytes}
                        }*/
                        let indent = formats.indentation;
                        formats.increase_indentation();
                        let ty = types.get_type(t).unwrap();
                        let num_of_bytes = ty.get_bytes(types);
                        let data = ty.get_data(types);
                        stream.write(format!("for(int i = 0; out < len; ++i) {{\n").as_bytes())?;
                        for (i, _) in data.iter().enumerate() {
                            for _ in 0..indent {
                                stream.write("\t".as_bytes())?;
                            }
                            stream.write(
                                format!(
                                    "in[{} * {} + i] = out[{} + {} * i];",
                                    i, num_of_bytes, i, num_of_bytes,
                                )
                                .as_bytes(),
                            )?;
                        }
                        stream.write(format!("out += {}", num_of_bytes).as_bytes())?;
                        AssembledInstruction::Semicolon.write(formats, types, stream)?;
                        AssembledInstruction::EndBlock.write(formats, types, stream)?;
                    }
                    _ => {} // TODO
                }
            }
            //To
            ValueFormat::StreamOfElements => {
                let top = formats.out_format_stack.len();
                //From
                match formats.out_format_stack.get(top).unwrap() {
                    ValueFormat::StreamOfValues => {
                        /*for(int i = 0; out < len; ++i) {
                            in[{variable_index} + {new_t.num_of_bytes} * i] =
                                out[{variable_index} * {old_t.num_of_bytes} + i];
                            out += {old_t.num_of_bytes}
                        }*/
                        let indent = formats.indentation;
                        formats.increase_indentation();
                        let ty = types.get_type(t).unwrap();
                        let num_of_bytes = ty.get_bytes(types);
                        stream.write(format!("for(int i = 0; out < len; ++i) {{\n").as_bytes())?;
                        for (i, _) in ty.get_data(types).iter().enumerate() {
                            for _ in 0..indent {
                                stream.write("\t".as_bytes())?;
                            }
                            stream.write(
                                format!(
                                    "in[{} * {} + i] = out[{} + {} * i];",
                                    i, num_of_bytes, i, num_of_bytes,
                                )
                                .as_bytes(),
                            )?;
                        }
                        stream.write(format!("out += {}", num_of_bytes).as_bytes())?;
                        AssembledInstruction::Semicolon.write(formats, types, stream)?;
                        AssembledInstruction::EndBlock.write(formats, types, stream)?;
                    }
                    _ => {} // TODO
                }
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClassReference {
    This,
    ThisAs(u16),
}

pub struct AssemblyFormatContext {
    pub indentation: u8,

    in_class_id_stack: Vec<u16>,
    out_class_id_stack: Vec<u16>,
    in_format_stack: Vec<ValueFormat>,
    out_format_stack: Vec<ValueFormat>,
}
impl AssemblyFormatContext {
    pub fn new() -> AssemblyFormatContext {
        AssemblyFormatContext {
            indentation: 0u8,
            in_class_id_stack: Vec::new(),
            out_class_id_stack: Vec::new(),
            in_format_stack: Vec::new(),
            out_format_stack: Vec::new(),
        }
    }

    pub fn increase_indentation(&mut self) {
        self.indentation += 1;
    }

    pub fn in_class(&self, rf: &ClassReference) -> &u16 {
        let top = self.in_class_id_stack.len() - 1;
        match rf {
            ClassReference::This => self.in_class_id_stack.get(top).unwrap(),
            ClassReference::ThisAs(_) => self.in_class_id_stack.get(top).unwrap(),
        }
    }

    pub fn out_class(&self, rf: &ClassReference) -> &u16 {
        let top = self.out_class_id_stack.len() - 1;
        match rf {
            ClassReference::This => self.out_class_id_stack.get(top).unwrap(),
            ClassReference::ThisAs(_) => self.out_class_id_stack.get(top).unwrap(),
        }
    }

    pub fn push(&mut self, t: &u16, format: ValueFormat) {
        self.in_class_id_stack.push(*t);
        self.out_class_id_stack.pop();
        self.in_format_stack.push(format);
        self.out_format_stack.pop();
    }

    pub fn pop(&mut self, t: &u16, format: ValueFormat) {
        self.in_class_id_stack.pop();
        self.out_class_id_stack.push(*t);
        self.in_format_stack.pop();
        self.out_format_stack.push(format);
    }

    pub fn already_in_format(&self, format: &ValueFormat) -> bool {
        let top = self.out_format_stack.len() - 1;
        *self.out_format_stack.get(top).unwrap() == *format
    }
}
pub struct AssembledTypeContext {
    types: HashMap<u16, AssembledType>,
    type_names: HashMap<String, u16>,
    impls: HashMap<u16, HashMap<String, AssembledMethod>>,
}
impl AssembledTypeContext {
    pub fn new() -> AssembledTypeContext {
        AssembledTypeContext {
            types: HashMap::new(),
            type_names: HashMap::new(),
            impls: HashMap::new(),
        }
    }

    pub fn get_impl_name(&self, type_name: &String) -> Option<&HashMap<String, AssembledMethod>> {
        self.impls.get(self.type_names.get(type_name).unwrap())
    }
    pub fn get_impl(&self, type_id: &u16) -> Option<&HashMap<String, AssembledMethod>> {
        self.impls.get(type_id)
    }
    pub fn record(&mut self, types: HashMap<String, u16>) {
        self.type_names = types;
    }
    pub fn setup_root_scope_types(&mut self) {
        self.types.insert(
            CLASS_ID_VOID,
            AssembledType::Interface(
                "()".to_string(),
                AssembledInterface::new(CLASS_ID_VOID, CLASS_ID_VOID),
            ),
        );
        self.types.insert(
            CLASS_ID_PRINTABLE,
            AssembledType::Interface(
                "Printable".to_string(),
                AssembledInterface::new(CLASS_ID_PRINTABLE, CLASS_ID_PRINTABLE),
            ),
        );
        let mut printable_impls = HashMap::new();
        let print_method = AssembledMethod::new(
            "print".to_string(),
            ValueFormat::StreamOfElements,
            ValueFormat::StreamOfElements,
            CLASS_ID_VOID,
        );
        printable_impls.insert("print".to_string(), print_method);
        self.types.insert(
            CLASS_ID_INT,
            AssembledType::Data("int".to_string(), AssembledData::filled(1u16, 0, 4)),
        );
        let mut int_impls = HashMap::new();
        let mut print_method = AssembledMethod::new(
            "print".to_string(),
            ValueFormat::StreamOfElements,
            ValueFormat::StreamOfElements,
            CLASS_ID_VOID,
        );
        print_method.add_instruction(AssembledInstruction::PrintValue(
            "%d".to_string(),
            Box::new(AssembledInstruction::Chain(
                Box::new(AssembledInstruction::Deref(Some("int".to_string()))),
                Box::new(AssembledInstruction::Get(ClassReference::This)),
            )),
        ));
        int_impls.insert("print".to_string(), print_method);

        let mut cast_method = AssembledMethod::new(
            "cast".to_string(),
            ValueFormat::StreamOfElements,
            ValueFormat::StreamOfElements,
            CLASS_ID_INT,
        );
        cast_method.add_instruction(AssembledInstruction::Chain(
            Box::new(AssembledInstruction::Deref(Some("int".to_string()))),
            Box::new(AssembledInstruction::Get(ClassReference::This)),
        ));
        int_impls.insert("cast".to_string(), cast_method);
        self.impls.insert(CLASS_ID_INT, int_impls);

        let mut obj_impls = HashMap::new();
        let mut print_method = AssembledMethod::new(
            "print".to_string(),
            ValueFormat::StreamOfElements,
            ValueFormat::StreamOfElements,
            CLASS_ID_VOID,
        );
        print_method.add_instruction(AssembledInstruction::Stream(
            ClassReference::ThisAs(CLASS_ID_PRINTABLE),
            StreamInstruction::ForEach(Box::new(AssembledInstruction::CallMethod(
                ClassReference::ThisAs(CLASS_ID_PRINTABLE),
                "print".to_string(),
            ))),
        ));
        obj_impls.insert("print".to_string(), print_method);

        self.impls.insert(CLASS_ID_VOID, obj_impls);
    }

    fn next_class_id(&self) -> u16 {
        self.types.len() as u16
    }

    pub fn create_type(&mut self, identifier: &String) -> u16 {
        let id = self.next_class_id();
        self.types.insert(
            id,
            AssembledType::Composition(identifier.clone(), AssembledComposition::new(id)),
        );
        self.type_names.insert(identifier.clone(), id);
        id
    }

    pub fn get_mut_type(&mut self, id: u16) -> Option<&mut AssembledType> {
        self.types.get_mut(&id)
    }
    pub fn get_type(&self, id: &u16) -> Option<&AssembledType> {
        self.types.get(id)
    }
    pub fn get_type_id(&self, name: &String) -> Option<&u16> {
        self.type_names.get(name)
    }

    pub fn add_variable(
        &mut self,
        _scope: usize,
        identifier: &String,
        variable: AssembledObjectAcceptor,
    ) -> Result<(), AssemblyError> {
        let id = self.type_names.get(identifier).unwrap();
        if let AssembledType::Composition(_, data) = self.types.get_mut(id).unwrap() {
            data.add_variable(variable);
            Ok(())
        } else {
            Err(AssemblyError::TriedToAddVariableToData)
        }
    }
}
