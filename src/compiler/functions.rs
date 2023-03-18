use crate::utils::UpValue;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct FunctionData {
    pub adress: usize,
    pub args_count: u32,
    pub is_native: bool,
    pub id: usize,
    pub upvalues: Vec<UpValue>,
}

impl FunctionData {
    fn new(adress: usize, args_count: u32) -> FunctionData {
        FunctionData {
            adress,
            args_count,
            is_native: false,
            id: 0,
            upvalues: Vec::new(),
        }
    }
    fn new_native(args_count: u32, id: usize) -> FunctionData {
        FunctionData {
            adress: 0,
            args_count,
            is_native: true,
            id,
            upvalues: Vec::new(),
        }
    }

    pub fn add_up_value(&mut self, upvalue: UpValue) -> usize {
        self.upvalues.push(upvalue);
        self.upvalues.len() - 1
    }
}

pub struct FunctionTable {
    functions: HashMap<String, FunctionData>,
    top: usize,
    current: Vec<String>, // the names of the functions callframes we are inside of currently
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

    pub fn print_functions(&self) {
        for func in self.functions.iter() {
            println!("{:?}", func);
        }
    }

    pub fn get(&self, key: &String) -> Option<&FunctionData> {
        self.functions.get(key)
    }

    pub fn get_mut(&mut self, key: &String) -> Option<&mut FunctionData> {
        self.functions.get_mut(key)
    }

    pub fn get_mut_last(&mut self) -> Option<&mut FunctionData> {
        let key = self.current.last().unwrap().clone();
        self.get_mut(&key)
    }

    fn get_mut_from_back(&mut self, offset: usize) -> Option<&mut FunctionData> {
        let key_opt;
        if let Some(key) = self.current.get(self.current.len() - offset).clone() {
            key_opt = key.clone();
        } else {
            return None;
        }
        self.get_mut(&key_opt)
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

    // We return the index of the added upvalue, meaning at runtime when this upvalue needs to be
    // accessed it will be in the callframe.upvalues at index i
    //
    // arguments:
    // Now the index tells us where the local variable sits in regards to the callframe_distance
    // Say we have
    // fn t() {
    //  let x = 0;
    //  fn tt() => x; (1)
    //  return tt;
    // }
    //
    // Then when compiling x in (1) the callframe_distance is 1 as x is not in the callframe of tt
    // but in the callframe of t which is 1 away
    pub fn add_up_value(&mut self, index: usize, mut callframe_distance: u32) -> usize {
        if callframe_distance == 1 {
            // If the distance is only one we can add the upvalue directly
            // So get the last function, so the function that is currently being compiled and add
            // the upvalue to it.
            // Then return the index of the upvalue of this function.
            if let Some(current_function) = self.get_mut_last() {
                // the upvalue is now the local at index index one callframe above.
                let upvalue = UpValue::Local(index);
                return current_function.add_up_value(upvalue);
            } else {
                0 // TODO
            }
        } else {
            // In this case we must go up more then once so
            // So we first add this upvalue to the function that has to acutally capture the value
            // for example 
            // fn t() {
            //  let x = 0;
            //  fn tt() {
            //      fn ttt() => x;
            //      return ttt;
            //  }
            //  return tt;
            // }
            //
            // here not ttt has to capture x, but tt.
            
            // Gets the topmost function
            let mut pointer;
            if let Some(func) = self.get_mut_from_back(callframe_distance as usize) {
                let upvalue = UpValue::Local(index);
                // this is now a pointer to the actuall upvalue so this
                pointer = func.add_up_value(upvalue);
                callframe_distance -= 1;
            } else {
                println!("Couldnt resolve upvalue");
                return 0;
            }

            while callframe_distance >= 1 {
                // Here we add the Recursive upvalues.
                if let Some(function) = self.get_mut_from_back(callframe_distance as usize) {
                    pointer = function.add_up_value(UpValue::Recursive(pointer));
                }
                callframe_distance -= 1;
            }
            return pointer;
        }
        
    }

    pub fn enter_function(&mut self, name: String) {
        self.current.push(name);
    }

    pub fn exit_function(&mut self) {
        self.current.pop();
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
