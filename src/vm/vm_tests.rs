use crate::vm::chunk::*;
use crate::vm::core::interpret;
use crate::vm::instructions::*;
use crate::vm::values::*;

#[test]
fn constant() {
    let mut chunk = Chunk::new();

    let c = chunk.push_constant(Constant::Float(20.0));
    let c2 = chunk.push_constant(Constant::Float(40.0));

    chunk.push_instruction(Instruction::Constant(c));
    chunk.push_instruction(Instruction::Constant(c2));
    chunk.push_instruction(Instruction::Add);
    chunk.push_instruction(Instruction::DEBUG);

    chunk.print_code();
    chunk.print_constants();

    let _ = interpret(chunk);
}

#[test]
fn power_float() {
    let mut chunk = Chunk::new();

    let c = chunk.push_constant(Constant::Float(2.0));
    let c2 = chunk.push_constant(Constant::Float(5.0));

    chunk.push_instruction(Instruction::Constant(c));
    chunk.push_instruction(Instruction::Constant(c2));
    chunk.push_instruction(Instruction::Pow);
    chunk.push_instruction(Instruction::DEBUG);

    chunk.print_code();
    chunk.print_constants();

    let _ = interpret(chunk);
}

#[test]
fn negate_float() {
    let mut chunk = Chunk::new();

    let c = chunk.push_constant(Constant::Float(2.0));

    chunk.push_instruction(Instruction::Constant(c));
    chunk.push_instruction(Instruction::Negate);
    chunk.push_instruction(Instruction::DEBUG);

    chunk.print_code();
    chunk.print_constants();

    let _ = interpret(chunk);
}
