use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FunctionData {
    pub adress: usize,
    pub args_count: u32,
    pub is_native: bool,
    pub id: usize,
}

impl FunctionData {
    fn new(adress: usize, args_count: u32) -> FunctionData {
        FunctionData {
            adress,
            args_count,
            is_native: false,
            id: 0,
        }
    }
    fn new_native(args_count: u32, id: usize) -> FunctionData {
        FunctionData {
            adress: 0,
            args_count,
            is_native: true,
            id,
        }
    }
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionData>,
    top: usize,
}

impl FunctionTable {
    pub fn new() -> FunctionTable {
        FunctionTable {
            functions: HashMap::new(),
            top: 0,
        }
    }

    pub fn get(&self, key: &String) -> Option<&FunctionData> {
        match self.functions.get(key) {
            Some(p) => Some(p),
            None => None,
        }
    }

    pub fn put(&mut self, key: String, adress: usize, args_count: u32) -> usize {
        self.functions
            .insert(key, FunctionData::new(adress, args_count));
        self.top += 1;
        self.top - 1
    }

    pub fn add_native(&mut self, key: String, id: usize, args_count: u32) -> usize {
        self.functions
            .insert(key, FunctionData::new_native(args_count, id));
        self.top += 1;
        self.top - 1
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
//
