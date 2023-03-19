use crate::vm::chunk::Chunk;

#[derive(Debug)]
pub struct Locals {
    locals: Vec<Local>,
    scope_depth: u32,
    local_call_fame_offsets: Vec<usize>,
}

impl Locals {
    pub fn new() -> Locals {
        Locals {
            scope_depth: 0,
            locals: Vec::new(),
            local_call_fame_offsets: Vec::new(),
        }
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    pub fn end_scope(&mut self, chunk: &mut Chunk) {
        self.scope_depth -= 1;
        while let Some(local) = self.locals.pop() {
            if local.scope_depth <= self.scope_depth {
                self.locals.push(local);
                break;
            } else {
                chunk.push_instruction(crate::vm::instructions::Instruction::Pop);
            }
        }
    }

    pub fn is_global_scope(&self) -> bool {
        self.scope_depth == 0
    }

    pub fn add_local(&mut self, name: String) {
        println!("{:?}", name);
        self.locals.push(Local {
            name,
            scope_depth: self.scope_depth,
            callframe_depth: self.local_call_fame_offsets.len() as u32,
        });
    }

    pub fn get_local(&self, name: &String) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate() {
            if name == &local.name {
                if self.local_call_fame_offsets.is_empty() {
                    return Some(i);
                }
                return match self.local_call_fame_offsets.last() {
                    Some(offset) => {
                        if *offset > i {
                            // This means the local is a upvalue as it is not in
                            // the callframe.
                            None
                        } else {
                            Some(i - offset)
                        }
                    }
                    None => Some(i),
                };
            }
        }
        None
    }
    // Checks if the variable with name can be found in a callframe
    pub fn get_upvalue(&self, name: &String) -> Option<(usize, u32)> {
        for (i, local) in self.locals.iter().enumerate() {
            if name == &local.name {
                let call_frame_offset =
                    self.local_call_fame_offsets[local.callframe_depth as usize - 1];
                return Some((
                    i - call_frame_offset, // i is the real place on the stack, call_frame_offset
                    // is the offset that is calculated at runtime to get the real pointer into the
                    // stack
                    self.local_call_fame_offsets.len() as u32 - local.callframe_depth, // this
                    // tells us how many call frames we would have to go up to find this variable
                ));
            }
        }
        None
    }

    pub fn new_function(&mut self) {
        self.local_call_fame_offsets.push(self.locals.len());
    }
    pub fn end_function(&mut self) {
        if !self.local_call_fame_offsets.is_empty() {
            let desired_stack_height = self.local_call_fame_offsets.pop().unwrap();
            while self.locals.len() > desired_stack_height {
                self.locals.pop();
            }
        }
    }
}
#[derive(Debug)]
pub struct Local {
    name: String,
    scope_depth: u32,
    // this represents in which callframe_depth the local was declared
    // for example
    //
    // fn test() {
    //  let x = 10;
    //  {
    //  let y = 20;
    //  }
    //  fn test2() {
    //      let z = 30;
    //      y ++; // callframe_depth = 2
    //  }
    // }
    // x has callframe_depth 1 as it is declared inside a function,
    // y has callframe_depth 1 but a different scope_depth
    // z has callframe_depth 2 as it is declared one function declaration deeper.
    //
    callframe_depth: u32,
    // type: TypeInformation // if adding type checking
}
