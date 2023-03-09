use crate::lexing::lexer::Token;
use crate::utils::LangError;
use crate::vm::chunk::Chunk;
use crate::vm::instructions::Instruction;
use crate::vm::values::{Constant, Value};
use crate::vm::compiler::Compiler;

pub fn compile(tokens: Vec<Token>) -> Result<Chunk, LangError> {
    let compiler = Compiler::new(tokens);
    compiler.compile()
}

pub fn interpret(chunk: Chunk) -> Result<(), LangError> {
    let mut interpreter = Interpreter::new(chunk);
    interpreter.run()
}

struct Interpreter {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl Interpreter {
    fn new(chunk: Chunk) -> Interpreter {
        Interpreter {
            chunk: chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn error(&self, message: &str) -> Result<(), LangError> {
        println!("{}: {}", self.ip, message);
        Err(LangError::Runtime)
    }

    fn pop(&mut self) -> Option<Value> {
        match self.stack.pop() {
            Some(x) => Some(x),
            None => {
                let _ = self.error("Stack is empty");
                None
            }
        }
    }

    fn run(&mut self) -> Result<(), LangError> {
        loop {
            if self.ip >= self.chunk.code.len() {
                break;
            }

            let instruction = self.chunk.code[self.ip].clone();
            self.ip += 1;

            match instruction {
                Instruction::DEBUG => println!("{:?}", self.pop().unwrap()),
                Instruction::Return => {
                    return Ok(());
                }
                Instruction::Constant(c) => {
                    let constant = &self.chunk.constants[c];
                    self.push(constant.to_value());
                }
                Instruction::Negate => {
                    if let Some(x) = self.pop() {
                        match instruction.unary_op(x) {
                            Ok(res) => self.push(res),
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    } else {
                        return self.error("Negate op failed");
                    }
                }
                Instruction::Add
                | Instruction::Div
                | Instruction::Sub
                | Instruction::Mult
                | Instruction::Mod
                | Instruction::Pow => {
                    let left;
                    let right;

                    if let Some(r) = self.pop() {
                        right = r;
                    } else {
                        return self.error("Binary op failed: left operand.");
                    }

                    if let Some(l) = self.pop() {
                        left = l;
                    } else {
                        return self.error("Binary op failed: right operand");
                    }
                    match instruction.binary_op(left, right) {
                        Ok(res) => self.push(res),
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
