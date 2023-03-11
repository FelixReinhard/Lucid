use crate::utils::{LangError, Value, Constant};
use crate::lexer::{Token, TokenData};
use crate::vm::chunk::Chunk;
use crate::compiler::globaltable::GlobalTable;
use crate::compiler::tokenstream::TokenStream;
use crate::compiler::error::ErrorHandler;
use crate::vm::instructions::Instruction;
use std::collections::VecDeque;

pub fn compile(tokens: VecDeque<Token>) -> Option<Chunk> {
    let mut token_stream = TokenStream::new(tokens);
    let compiler = Compiler::new();
    compiler.compile(&mut token_stream)
}

pub struct Compiler {
    pub chunk: Chunk,
    pub globals: GlobalTable,
    pub error_handler: ErrorHandler,
}

impl Compiler {
    fn new() -> Compiler {
        let mut chunk = Chunk::new();
        chunk.push_constant(Constant::Bool(true));
        chunk.push_constant(Constant::Bool(false));
        chunk.push_constant(Constant::Null);

        Compiler{chunk, globals: GlobalTable::new(), error_handler: ErrorHandler::new()}
    }

    fn compile(mut self, tokens: &mut TokenStream) -> Option<Chunk> {
        
        while tokens.peek_not_eq(TokenData::EOF) && self.error_handler.can_continue() {
            self.declaration(tokens);
        }

        if self.error_handler.ok() {
            Some(self.chunk)
        } else {
            None
        }
    }

    pub fn emit(&mut self, instruction: Instruction) {
        let _ = self.emit_get(instruction);
    }

    pub fn emit_get(&mut self, instruction: Instruction) -> usize {
        self.chunk.push_instruction(instruction)
    }

    pub fn push_constant(&mut self, constant: Constant) -> usize {
        self.chunk.push_constant(constant)
    }
}
