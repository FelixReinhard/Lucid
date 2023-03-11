use crate::lexing::lexer::Token;
use crate::utils::{LangError, Value};
use crate::vm::chunk::Chunk;
use crate::vm::instructions::Instruction;

pub fn interpret(chunk: Chunk) -> Result<Value, LangError> {
    let mut interpreter = Interpreter::new(chunk);
    interpreter.run()
}

struct Interpreter {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: Vec<Value>,
}

impl Interpreter {
    fn new(chunk: Chunk) -> Interpreter {
        Interpreter {
            chunk,
            ip: 0,
            stack: Vec::new(),
            globals: Vec::new(),
        }
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn error(&self, message: &str) -> Result<Value, LangError> {
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

    fn peek(&self) -> Option<Value> {
        match self.stack.last() {
            Some(x) => Some(x.clone()),
            None => {
                let _ = self.error("Stack is empty cannot peek");
                None
            }
        }
    }

    fn run(&mut self) -> Result<Value, LangError> {
        loop {
            if self.ip >= self.chunk.code.len() {
                break;
            }

            let instruction = self.chunk.code[self.ip].clone();
            self.ip += 1;

            match instruction {
                Instruction::DEBUG => println!("{:?}", self.peek().unwrap().to_string()),
                Instruction::Return => {
                    return Ok(Value::Integer(0));
                }
                Instruction::DefGlobal(global) => {
                    let init_value;
                    if let Some(v) = self.pop() {
                        init_value = v;
                    } else {
                        return Err(LangError::RuntimeMessage(
                            "Error for getting init value of global variable",
                        ));
                    }
                    if self.globals.len() > global {
                        self.globals.remove(global);
                    }
                    self.globals.insert(global, init_value);
                }
                Instruction::GetGlobal(global) => {
                    if let Some(p) = self.globals.get(global) {
                        self.push(p.clone());
                    } else {
                        return Err(LangError::RuntimeMessage("Undefined Global"));
                    }
                }
                Instruction::SetGlobal(global) => {
                    let init_value;
                    if let Some(v) = self.peek() {
                        init_value = v;
                    } else {
                        return Err(LangError::RuntimeMessage(
                            "Error for getting init value of global variable",
                        ));
                    }
                    self.globals.remove(global);
                    self.globals.insert(global, init_value);
                }
                Instruction::Constant(c) => {
                    let constant = &self.chunk.constants[c];
                    self.push(constant.to_value());
                }
                Instruction::Pop => {
                    let _ = self.pop();
                }
                Instruction::Negate | Instruction::Not => {
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
                | Instruction::Pow
                | Instruction::LogicOr
                | Instruction::LogicAnd
                | Instruction::Equal
                | Instruction::Greater
                | Instruction::Less
                | Instruction::ShiftRight
                | Instruction::ShiftLeft
                | Instruction::BitAnd
                | Instruction::BitOr => {
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
        if self.stack.is_empty() {
            Ok(Value::Null)
        } else {
            Ok(self.stack.pop().unwrap())
        }
    }
}
