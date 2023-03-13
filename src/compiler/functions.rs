use std::collections::HashMap;

#[derive(Clone)]
pub struct FunctionData {
    pub adress: usize,
    pub args_count: u32,
}

impl FunctionData {
    fn new(adress: usize, args_count: u32) -> FunctionData {
        FunctionData{adress, args_count}
    }
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionData>,
    top: usize,
}

impl FunctionTable {
    pub fn new() -> FunctionTable {
        FunctionTable{functions: HashMap::new(), top: 0}
    }

    pub fn get(&self, key: &String) -> Option<&FunctionData> {
        match self.functions.get(key) {
            Some(p) => Some(p),
            None => None,
        }
    }

    pub fn put(&mut self, key: String, adress: usize, args_count: u32) -> usize {
        self.functions.insert(key, FunctionData::new(adress, args_count));
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
