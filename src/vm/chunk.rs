use crate::vm::instructions::Instruction;
use crate::utils::Constant;

#[derive(Debug)]
pub struct Chunk {
    pub code: Vec<Instruction>,
    pub constants: Vec<Constant>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn push_instruction(&mut self, instruction: Instruction) -> usize {
        self.code.push(instruction);
        self.code.len() - 1
    }

    pub fn patch_instruction(&mut self, slot: usize, instruction: Instruction) -> bool {
        if self.code.len() <= slot {
            false 
        } else {
            self.code[slot] = instruction;
            true
        }
    }

    pub fn push_constant(&mut self, constant: Constant) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    pub fn print_code(&self) {
        for (i, instruction) in self.code.iter().enumerate() {
            println!("{}: {:?}", i, instruction);
        }
        println!();
    }

    pub fn print_constants(&self) {
        for (i, constant) in self.constants.iter().enumerate() {
            println!("{}: {:?}", i, constant);
        }
        println!();
    }
}
