use crate::utils::{LangError, Value};
use crate::vm::chunk::Chunk;
use crate::vm::instructions::Instruction;
use crate::vm::native::execute_native_function;
use crate::compiler::functions::UpValue;

use std::rc::Rc;
use std::cell::RefCell;

pub fn interpret(chunk: Chunk, print_stack: bool) -> Result<Value, LangError> {
    let mut interpreter = Interpreter::new(chunk);
    interpreter.run(print_stack)
}

#[derive(Debug)]
struct CallFrame {
    return_adress: usize,
    ip_offset: usize,
    args_count: u32,
    locals_to_pop: u32,
    up_values: Vec<Value>,
}

impl CallFrame {
    fn new(
        return_adress: usize,
        ip_offset: usize,
        args_count: u32,
        locals_to_pop: u32,
        up_values: Vec<Value>,
    ) -> CallFrame {
        CallFrame {
            return_adress,
            ip_offset,
            args_count,
            locals_to_pop,
            up_values,
        }
    }
}

struct Interpreter {
    chunk: Chunk,
    debug_value: Value,
    ip: usize,
    call_frames: Vec<CallFrame>, // similar to $re in mips, when function is called adress is pushed
    // and when returning adress is poped.
    stack: Vec<Value>,
    globals: Vec<Value>,
}

impl Interpreter {
    fn new(chunk: Chunk) -> Interpreter {
        let mut call_frames: Vec<CallFrame> = Vec::new();
        call_frames.push(CallFrame::new(0, 0, 0, 0, Vec::new()));
        Interpreter {
            chunk,
            ip: 0,
            call_frames,
            debug_value: Value::Null,
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

    fn get_absolute_pointer(&self, offset: usize) -> usize {
        self.call_frames.last().unwrap().ip_offset + offset
    }

    fn resolve_upvalues(&self, up_values: &Box<Rc<RefCell<Vec<UpValue>>>>) -> Vec<Value> {
        let mut values = Vec::new();
        let borrow = up_values.borrow_mut();

        let up_values_iter = borrow.iter();
        for up_value in up_values_iter {
            // push a copy of the captured value onto the up_value that reside inside the function
            // value (Note: Not the reference, as each 'instance' of the funcref can capture
            // the same variable, but this variable may have a different runtime value).
            values.push(self.stack[self.get_absolute_pointer(up_value.index)].clone());
        }

        values
    }

    fn run(&mut self, print_stack: bool) -> Result<Value, LangError> {
        loop {
            if self.ip >= self.chunk.code.len() {
                break;
            }

            let instruction = self.chunk.code[self.ip].clone();
            self.ip += 1;
            if print_stack {
                println!("IP: {}, STACK: {:?}", self.ip - 1, self.stack);
            }
            match instruction {
                Instruction::DEBUG => {
                    let v = self.pop().unwrap_or(Value::Null);
                    println!("Debug : {:?}", v.to_string());
                    self.debug_value = v;
                }
                Instruction::JumpIfFalse(amount) => match self.peek() {
                    Some(val) => {
                        if let Some(jump) = val.is_falsey() {
                            self.ip += if jump { amount } else { 0 };
                        } else {
                            return Err(LangError::RuntimeMessage(
                                "None boolean value in expression if statement",
                            ));
                        }
                    }
                    None => {
                        return Err(LangError::RuntimeMessage(
                            "Could not peek stack. Seems to be empty",
                        ))
                    }
                },
                Instruction::Jump(amount) => {
                    self.ip += amount;
                }
                Instruction::JumpTo(amount) => {
                    self.ip = amount;
                }
                Instruction::JumpRe => {
                    self.ip = self
                        .call_frames
                        .get(self.call_frames.len() - 1)
                        .unwrap()
                        .return_adress
                        - 1; // -1 as it gets increased after that.
                    self.call_frames.pop();
                }
                Instruction::Return => {
                    // get the return value of the function and save it for now.
                    if let Some(top) = self.pop() {
                        let call_frame = self.call_frames.pop();
                        match call_frame {
                            Some(frame) => {
                                while self.stack.len() >= frame.ip_offset {
                                    // pop off all locals and the funcref value
                                    self.pop();
                                }
                                self.ip = frame.return_adress - 1;
                                self.push(top);
                            }
                            _ => {
                                return Err(LangError::RuntimeMessage(
                                    "Couldnt return from function as no call frame is there",
                                ));
                            }
                        }
                    } else {
                        return Err(LangError::RuntimeMessage(
                            "Return could not get stack top value",
                        ));
                    }
                }
                Instruction::CallFunc(args_given) => {
                    let args;
                    if let Ok(a) = usize::try_from(args_given) {
                        args = a;
                    } else {
                        return Err(LangError::RuntimeMessage(
                            "Couldnt parse argcount (usize) to u32",
                        ));
                    }
                    if self.stack.len() < 1 + args {
                        return Err(LangError::RuntimeMessage("Perhaps you forgot a return"));
                    }

                    if let Value::Func(adress, args_count, up_values) = &self.stack[self.stack.len() - 1 - args]
                    {
                        if args_given != *args_count {
                            return Err(LangError::RuntimeMessage(
                                "Called function with wrong number of args",
                            ));
                        }
                        // create vec of upvalues, meaning copy the local variables that each
                        // upvalue refers to into the
                        let mut up_values_copies = Vec::new();

                        for val in up_values.iter() {
                            up_values_copies.push(val.clone());
                        }

                        self.call_frames.push(CallFrame::new(
                            self.ip + 1,
                            self.stack.len() - args,
                            *args_count,
                            0,
                            up_values_copies,
                        ));
                        self.ip = *adress;
                    } else if let Value::NativeFunc(id, _args_count) =
                        self.stack[self.stack.len() - 1 - args]
                    {
                        let mut args_list: Vec<Value> = Vec::new();
                        for _ in 0..args_given {
                            match self.pop() {
                                Some(v) => args_list.push(v),
                                None => {
                                    return Err(LangError::RuntimeMessage(
                                        "Wrong amount of arguments to native function",
                                    ));
                                }
                            }
                        }
                        self.pop();
                        match execute_native_function(id, args_list) {
                            Some(v) => self.push(v),
                            None => {
                                return Err(LangError::RuntimeMessage("error calling native func."))
                            }
                        }
                    } else {
                        return Err(LangError::RuntimeMessage("Cannot call none function type"));
                    }
                }
                // Push the value of the function onto the stack if followed by a CallFunc we pop
                // this value and execute it. 
                // Upvalues 
                // Consider this program
                //
                // fn outer() {
                //  let x = 10;
                //  fn inner() return x;
                //  return inner; (1)
                // }
                //
                // let res = outer()();
                //
                // res should be 10. 
                // To capture the x we must at (1) know that the function inner captures the local
                // variable x, So at (1) we copy x to the actuall Value::Func object and save it
                // there, then when calling it we add this value to the up_values of the call
                // frame. When inside the call of inner() instead of GetLocal we do GetUpvalue
                // which looks inside the call frame and returns the 0th upvalues. As it is know at
                // compile time which index the upvalue has compiling this is not a problem.
                Instruction::FuncRef(adress, args_count, up_values) => {
                    let up_value_objs = self.resolve_upvalues(&up_values);
                    self.push(Value::Func(adress, args_count, Box::new(up_value_objs)));
                }
                Instruction::GetUpvalue(index) => {
                    self.push(self.call_frames.last().unwrap().up_values[index].clone());
                }
                Instruction::SetUpvalue(index) => {
                    let last = self.call_frames.len() -1;
                    self.call_frames[last].up_values[index] = self.peek().unwrap();
                }
                Instruction::NativeRef(id, args_count) => {
                    self.push(Value::NativeFunc(id, args_count));
                }
                Instruction::GetLocal(pointer) => {
                    let pointer = self.get_absolute_pointer(pointer);
                    if self.stack.len() <= pointer {
                        return Err(LangError::RuntimeMessage("Couldnt get local"));
                    }
                    self.push(self.stack[pointer].clone());
                }
                Instruction::SetLocal(pointer) => {
                    let pointer = self.get_absolute_pointer(pointer);
                    if let Some(v) = self.peek() {
                        self.stack[pointer] = v;
                    } else {
                        return Err(LangError::RuntimeMessage("Could not set local variable"));
                    }
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
        Ok(self.debug_value.clone())
    }
}
