use std::collections::HashMap;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct UpValue {
    pub index: usize,
    pub is_local: bool,
}

#[derive(Clone, Debug)]
pub struct FunctionData {
    pub adress: usize,
    pub args_count: u32,
    pub is_native: bool,
    pub id: usize,
    pub upvalues: Rc<RefCell<Vec<UpValue>>>,
}

impl FunctionData {
    fn new(adress: usize, args_count: u32) -> FunctionData {
        FunctionData {
            adress,
            args_count,
            is_native: false,
            id: 0,
            upvalues: Rc::new(RefCell::new(Vec::new())),
        }
    }
    fn new_native(args_count: u32, id: usize) -> FunctionData {
        FunctionData {
            adress: 0,
            args_count,
            is_native: true,
            id,
            upvalues: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionData>,
    top: usize,
    current: Vec<String>,
    lambda_count: usize,
}

impl FunctionTable {
    pub fn new() -> FunctionTable {
        FunctionTable {
            functions: HashMap::new(),
            top: 0,
            current: Vec::new(),
            lambda_count: 0,
        }
    }

    pub fn get(&self, key: &String) -> Option<&FunctionData> {
        match self.functions.get(key) {
            Some(p) => Some(p),
            None => None,
        }
    }

    pub fn put(&mut self, key: String, adress: usize, args_count: u32) -> usize {
        self.enter_function(key.clone());
        self.functions
            .insert(key, FunctionData::new(adress, args_count));
        self.top += 1;
        self.top - 1
    }

    pub fn put_lambda(&mut self, adress: usize, args_count: u32) -> usize {
        let key = format!("{}", self.lambda_count);
        self.lambda_count += 1;

        self.enter_function(key.clone());
        self.functions
            .insert(key, FunctionData::new(adress, args_count));
        self.top += 1;
        self.lambda_count - 1
    }

    pub fn get_lambda(&self, key: usize) -> Option<&FunctionData> {
        match self.functions.get(&format!("{}", key)) {
            Some(p) => Some(p),
            None => None,
        }
    }

    pub fn add_native(&mut self, key: String, id: usize, args_count: u32) -> usize {
        self.functions
            .insert(key, FunctionData::new_native(args_count, id));
        self.top += 1;
        self.top - 1
    }

    pub fn enter_function(&mut self, name: String) {
        self.current.push(name);
    }

    pub fn exit_function(&mut self) {
        self.current.pop();
    }

    pub fn add_upvalue(&mut self, upvalue: UpValue) -> Option<usize> {
        if let Some(func) = self.functions.get_mut(self.current.last().unwrap()) {
            let mut reference = func.upvalues.borrow_mut();
            reference.push(upvalue);
            return Some(reference.len() -1);
        } 
        None
    }
}

// when calling a function for example: test(1, 2)
// with fn test(x, y) {
//      if x == y => return x;
//      else return 10;
// }
//
// first we evaluate 1 and 2 which leads to
// two constants sitting on top of the stack. lets say 1 sits at i and 2 at i
// We also need a return adress to jump to after the function is done
//
// so a call does
// - first get the current ip and push it onto the stack
// - then push all arguments onto the stack
// - begin a scope
// - jump to the functions code
//
