use crate::lang::assemble::context::{AssembledTypeContext, AssemblyFormatContext, ClassReference};

use std::fs;
use std::io::{BufWriter, Write};

#[derive(Debug)]
pub enum StreamInstruction {
    ForEach(Box<AssembledInstruction>),
}

#[derive(Debug)]
pub enum AssembledInstruction {
    DeclareContext,
    InitContext,
    DeclareCalculationScope,
    SetLength(u64),
    InitIntake(u64),
    InitOutgive(u64),
    LoadIntake(u16),
    SaveOutgive(u16),
    NoOp,
    AddIntake(Box<AssembledInstruction>, ClassReference),
    FlipIntake,
    PrintString(u64, Box<AssembledInstruction>), //id, len
    PrintValue(String, Box<AssembledInstruction>), // format, value
    CallMethod(ClassReference, String),
    Cast(String),
    Deref(Option<String>),
    Get(ClassReference),
    Stream(ClassReference, StreamInstruction),
    Multiply(Box<AssembledInstruction>, Box<AssembledInstruction>),
    Chain(Box<AssembledInstruction>, Box<AssembledInstruction>),
    EndBlock,
    Semicolon,
    Indent,
}

impl AssembledInstruction {
    pub fn write(
        &self,
        formats: &mut AssemblyFormatContext,
        types: &AssembledTypeContext,
        stream: &mut BufWriter<fs::File>,
    ) -> std::io::Result<()> {
        match self {
            AssembledInstruction::DeclareContext => {
                stream.write(format!("struct __CONTEXT__ {{\n",).as_bytes())?;
                stream.write(format!("\tvoid* [256] v\n",).as_bytes())?;
            }
            AssembledInstruction::InitContext => {
                stream.write(format!("struct __CONTEXT__ c;\n").as_bytes())?;
            }
            AssembledInstruction::DeclareCalculationScope => {
                stream.write(format!("void* in; void* out;\nint len;\n\n",).as_bytes())?;
            }
            AssembledInstruction::SetLength(length) => {
                stream.write(format!("len = {};\n", length).as_bytes())?;
            }
            AssembledInstruction::InitIntake(byte_size) => {
                stream.write(format!("in = malloc({});\n", byte_size).as_bytes())?;
            }
            AssembledInstruction::InitOutgive(byte_size) => {
                stream.write(format!("out = malloc({});\n", byte_size).as_bytes())?;
            }
            AssembledInstruction::LoadIntake(ptr_id) => {
                stream.write(format!("in = formats.v[{}];\n", ptr_id).as_bytes())?;
            }
            AssembledInstruction::SaveOutgive(ptr_id) => {
                stream.write(format!("context.v[{}] = out;\n", ptr_id).as_bytes())?;
            }
            AssembledInstruction::NoOp => {
                stream.write(format!("out = in;\n").as_bytes())?;
            }
            AssembledInstruction::AddIntake(value, class_id) => {
                {
                    let class_id = formats.in_class(class_id);
                    let t = types.get_type(class_id).unwrap();
                    stream.write(format!("*(({}*)in) = ", t.get_name()).as_bytes())?;
                }
                value.as_ref().write(formats, types, stream)?;
                stream.write(format!(";\n",).as_bytes())?;
                {
                    let class_id = formats.in_class(class_id);
                    let t = types.get_type(class_id).unwrap();
                    let num_of_bytes = t.get_bytes(types);
                    stream.write(format!("in += {};\n", num_of_bytes).as_bytes())?;
                    stream.write(format!("len += {};\n", num_of_bytes).as_bytes())?;
                }
            }
            AssembledInstruction::FlipIntake => {
                stream.write(format!("in -= len;\n").as_bytes())?;
            }
            AssembledInstruction::PrintString(len, value) => {
                stream.write(format!("printf(\"%.{}s\", ", len).as_bytes())?;
                value.as_ref().write(formats, types, stream)?;
                stream.write(format!(")").as_bytes())?;
            }
            AssembledInstruction::PrintValue(f, value) => {
                stream.write(format!("printf(\"{}\", \n", f,).as_bytes())?;
                value.as_ref().write(formats, types, stream)?;
                stream.write(format!(")").as_bytes())?;
            }
            AssembledInstruction::Cast(primitive_type) => {
                stream.write(format!("({} *)", primitive_type,).as_bytes())?;
            }
            AssembledInstruction::Deref(primitive_type_o) => {
                if let Some(primitive_type) = primitive_type_o {
                    stream.write(format!("*({} *)", primitive_type,).as_bytes())?;
                } else {
                    stream.write(format!("*",).as_bytes())?;
                }
            }
            AssembledInstruction::CallMethod(class_id, method_name) => {
                let class_id = *formats.out_class(class_id);
                let imp = types.get_impl(&class_id).unwrap();
                let method = imp.get(method_name).unwrap();
                method.write(&class_id, formats, types, stream)?;
            }
            AssembledInstruction::Multiply(op1, op2) => {
                stream.write(format!("(").as_bytes())?;
                op1.as_ref().write(formats, types, stream)?;
                stream.write(format!(" * ").as_bytes())?;
                op2.as_ref().write(formats, types, stream)?;
                stream.write(format!(")").as_bytes())?;
            }
            AssembledInstruction::Chain(op1, op2) => {
                op1.as_ref().write(formats, types, stream)?;
                op2.as_ref().write(formats, types, stream)?;
            }
            AssembledInstruction::Get(_) => {
                stream.write(format!("in",).as_bytes())?;
            }
            AssembledInstruction::Stream(class_id, _stream_instruction) => {
                let t = types.get_type(&formats.in_class(class_id)).unwrap();
                let num_of_bytes = t.get_bytes(types);
                AssembledInstruction::FlipIntake.write(formats, types, stream)?;
                stream.write(format!("while(in) {{",).as_bytes())?;
                formats.increase_indentation();
                AssembledInstruction::Indent.write(formats, types, stream)?;
                stream.write(format!("in += {};", num_of_bytes).as_bytes())?;
                AssembledInstruction::EndBlock.write(formats, types, stream)?;
            }
            AssembledInstruction::Indent => {
                for _ in 0..formats.indentation {
                    stream.write("\t".as_bytes())?;
                }
            }
            AssembledInstruction::EndBlock => {
                stream.write(format!("}}\n").as_bytes())?;
                AssembledInstruction::Indent.write(formats, types, stream)?;
            }
            AssembledInstruction::Semicolon => {
                if formats.indentation > 0 {
                    stream.write(format!(";\n").as_bytes())?;
                    for _ in 0..formats.indentation {
                        stream.write("\t".as_bytes())?;
                    }
                } else {
                    stream.write(format!(";\n").as_bytes())?;
                }
            }
        };
        Ok(())
    }
}
