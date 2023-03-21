use crate::utils::{LangError, List, UpValue, UpValueList, Value};
use crate::vm::chunk::Chunk;
use crate::vm::instructions::Instruction;
use crate::vm::native::execute_native_function;

use std::cell::RefCell;
use std::rc::Rc;

pub fn interpret(chunk: Chunk, print_stack: bool) -> Result<Value, LangError> {
    let mut interpreter = Interpreter::new(chunk);
    interpreter.run(print_stack)
}

#[derive(Debug)]
struct CallFrame {
    return_adress: usize,
    ip_offset: usize,
    up_values: List,
    selff: Value,
}

impl CallFrame {
    fn new(return_adress: usize, ip_offset: usize, up_values: List) -> CallFrame {
        CallFrame {
            return_adress,
            ip_offset,
            up_values,
            selff: Value::Null,
        }
    }

    // get the upvalue from the callframe and return a new shared value
    fn get_up_value(&self, index: usize) -> Option<Value> {
        if let Some(Value::Shared(v)) = self.up_values.borrow().get(index) {
            Some(Value::Shared(Box::new(Rc::clone(v))))
        } else {
            None
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
        call_frames.push(CallFrame::new(
            0,
            0,
            Rc::new(Box::new(RefCell::new(Vec::new()))),
        ));
        Interpreter {
            chunk,
            ip: 0,
            call_frames,
            debug_value: Value::Null,
            stack: Vec::new(),
            globals: Vec::new(),
        }
    }

    fn set_self(&mut self, val: Value) {
        if let Some(frame) = self.call_frames.last_mut() {
            frame.selff = val;
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

    fn capture_upvalues(&mut self, up_value_definitions: UpValueList) -> Option<List> {
        let definitions = up_value_definitions.borrow();
        let mut up_values: Vec<Value> = Vec::new();
        for def in definitions.iter() {
            match def {
                UpValue::Local(local_index) => {
                    // In this case the value that should be captured is a local variable so we
                    // resolve it the same way
                    let pointer = self.get_absolute_pointer(*local_index);
                    if self.stack.len() <= pointer {
                        return None;
                    }
                    // now replace the Value at that slot by a Shared Value
                    let value = Rc::new(RefCell::new(self.stack.remove(pointer)));
                    self.stack
                        .insert(pointer, Value::Shared(Box::new(Rc::clone(&value))));

                    up_values.push(Value::Shared(Box::new(Rc::clone(&value))));
                }
                UpValue::Recursive(call_frame_value) => {
                    // In this case we refer to an upvalue of the call_frame
                    let frame = self.call_frames.last().unwrap();
                    if let Some(v) = frame.get_up_value(*call_frame_value) {
                        up_values.push(v);
                    } else {
                        return None;
                    }
                }
            }
        }
        Some(Rc::new(Box::new(RefCell::new(up_values))))
    }

    fn run(&mut self, print_stack: bool) -> Result<Value, LangError> {
        loop {
            if self.ip >= self.chunk.code.len() {
                break;
            }

            let instruction = self.chunk.code[self.ip].clone();
            self.ip += 1;
            if print_stack {
                println!(
                    "IP: {}, STACK: [{}]",
                    self.ip - 1,
                    self.stack
                        .iter()
                        .map(|v| format!("{}, ", v.to_debug()))
                        .collect::<String>()
                );
            }
            match instruction {
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
                                match frame.selff {
                                    Value::StructInstance(_, _) => {
                                        self.pop();
                                    }
                                    _ => {}
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
                Instruction::Struct(map) => {
                    let mut values = Vec::new();
                    for _ in 0..map.len() {
                        if let Some(val) = self.pop() {
                            values.push(val);
                        } else {
                            return Err(LangError::RuntimeMessage("Couldnt pop"));
                        }
                    }
                    values.reverse();

                    self.push(Value::StructInstance(
                        Rc::new(Box::new(RefCell::new(values))),
                        map,
                    ));
                }
                Instruction::DefineSelf(offset) => {
                    if let Some(val) = self.stack.get(self.stack.len() - 1 - offset) {
                        self.set_self(val.clone());
                    } else {
                        return Err(LangError::RuntimeMessage("Couldnt pop for def self"));
                    }
                }
                Instruction::GetSelf => {
                    if let Some(frame) = self.call_frames.last() {
                        self.push(frame.selff.clone());
                    } else {
                        return Err(LangError::RuntimeMessage("No selff here"));
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

                    if let Value::Func(adress, args_count, up_vals) =
                        &self.stack[self.stack.len() - 1 - args]
                    {
                        if args_given != *args_count {
                            return Err(LangError::RuntimeMessage(
                                "Called function with wrong number of args",
                            ));
                        }

                        self.call_frames.push(CallFrame::new(
                            self.ip + 1,
                            self.stack.len() - args,
                            Rc::clone(up_vals),
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
                Instruction::GetUpvalue(index) => {
                    let val = self.call_frames.last().unwrap().up_values.borrow()[index].clone();
                    self.push(val);
                }
                Instruction::SetUpvalue(index) => {
                    if let Some(val) = self.peek() {
                        let ref mut upvalue =
                            *self.call_frames.last().unwrap().up_values.borrow_mut();
                        if let Value::Shared(ref mut old) = upvalue[index] {
                            old.replace_with(|&mut _| val);
                        }
                    } else {
                        return Err(LangError::RuntimeMessage(
                            "Cannot set Upvalue, no value there",
                        ));
                    }
                }
                Instruction::FuncRef(adress, args_count, up_value_definitions) => {
                    if let Some(captured_values) = self.capture_upvalues(up_value_definitions) {
                        self.push(Value::Func(adress, args_count, captured_values));
                    } else {
                        return Err(LangError::RuntimeMessage("Funcref coulndt get upvals"));
                    }
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
                Instruction::DefList(init_amount) => {
                    let mut ls = Vec::new();
                    for _ in 0..init_amount {
                        if let Some(val) = self.pop() {
                            ls.push(val);
                        } else {
                            return Err(LangError::RuntimeMessage(
                                "Could not init list, not enough args",
                            ));
                        }
                    }
                    ls.reverse();
                    self.push(Value::List(Rc::new(Box::new(RefCell::new(ls)))));
                }
                Instruction::SetList => {
                    // let new_val = self.pop();
                    // let index = self.pop();
                    // let list_val = self.pop();
                    match (self.pop(), self.pop(), self.peek()) {
                        (Some(new_val), Some(Value::Integer(index)), Some(Value::List(ls_vec))) => {
                            let mut borrow = ls_vec.borrow_mut();
                            borrow[index as usize] = new_val;
                        }
                        _ => {
                            return Err(LangError::RuntimeMessage(
                                "Error while trying to set list element",
                            ));
                        }
                    }
                }
                Instruction::AccessList => {
                    if let Some(Value::Integer(index)) = self.pop() {
                        if let Some(Value::List(ls)) = self.pop() {
                            let borrow = ls.borrow();
                            if let Some(val) = borrow.get(index as usize) {
                                self.push(val.clone());
                                continue;
                            }
                        }
                    }
                    return Err(LangError::RuntimeMessage(
                        "Could not pop integer for array access",
                    ));
                }
                Instruction::StructGet(name) => {
                    let popped = self.pop();
                    if let Some(Value::StructInstance(values, names)) = popped {
                        let value = values.borrow()[*names.get(&*name).unwrap()].clone();
                        if let Value::Func(_, _, _) = value {
                            self.push(Value::StructInstance(values, names));
                        }
                        self.push(value);
                    } else {
                        return Err(LangError::RuntimeMessage("Could not pop struct for get"));
                    }
                }
                Instruction::StructSet(name) => {
                    let val = self.pop().unwrap();
                    if let Some(Value::StructInstance(values, names)) = self.peek() {
                        let mut borrow = values.borrow_mut();
                        borrow[*names.get(&*name).unwrap()] = val;
                    } else {
                        return Err(LangError::RuntimeMessage("Could not pop struct for get"));
                    }
                }
                Instruction::Dup(amount) => {
                    for i in self.stack.len() - amount..self.stack.len() {
                        self.push(self.stack[i].clone());
                    }
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
