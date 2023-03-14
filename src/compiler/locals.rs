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
            local_call_fame_offsets : Vec::new(),
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
        self.locals.push(Local{name, scope_depth: self.scope_depth});
    }

    pub fn get_local(&self, name: &String) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate() {
            if name == &local.name {
                if self.local_call_fame_offsets.len() == 0 {
                    return Some(i);
                }
                return match self.local_call_fame_offsets.get(self.local_call_fame_offsets.len() - 1) {
                    Some(offset) => Some(i - offset),
                    None => Some(i)
                };
            }
        }
        None
    }

    pub fn new_function(&mut self) {
        if self.locals.len() > 0 {
            self.local_call_fame_offsets.push(self.locals.len());
        }
    }
    pub fn end_function(&mut self) {
        if self.local_call_fame_offsets.len() > 0 {
            self.local_call_fame_offsets.pop();
        }
    }
}
#[derive(Debug)]
pub struct Local {
    name: String,
    scope_depth: u32,
    // type: TypeInformation // if adding type checking
}
