use crate::compiler::error::ErrorHandler;
use crate::compiler::functions::FunctionTable;
use crate::compiler::globaltable::GlobalTable;
use crate::compiler::structs::StructTable;
use crate::compiler::locals::Locals;
use crate::compiler::tokenstream::TokenStream;
use crate::lexer::{Token, TokenData};
use crate::utils::Constant;
use crate::vm::chunk::Chunk;
use crate::vm::instructions::Instruction;
use std::collections::VecDeque;

pub fn compile(tokens: VecDeque<Token>, print_toks: bool) -> Option<Chunk> {
    let mut token_stream = TokenStream::new(tokens);
    let compiler = Compiler::new(print_toks);
    compiler.compile(&mut token_stream)
}

pub struct Compiler {
    chunk: Chunk,
    pub globals: GlobalTable,
    pub error_handler: ErrorHandler,
    pub locals: Locals,
    pub functions: FunctionTable,
    pub structs: StructTable,
    pub for_loop_count: u32,
    pub print_toks: bool,
}

impl Compiler {
    fn new(print_toks: bool) -> Compiler {
        let mut chunk = Chunk::new();
        chunk.push_constant(Constant::Bool(true));
        chunk.push_constant(Constant::Bool(false));
        chunk.push_constant(Constant::Null);

        Compiler {
            chunk,
            globals: GlobalTable::new(),
            error_handler: ErrorHandler::new(),
            locals: Locals::new(),
            functions: FunctionTable::new(),
            structs: StructTable::new(),
            for_loop_count: 0,
            print_toks,
        }
        .define_natives()
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

    pub fn compile_import(&mut self, tokens: &mut TokenStream) {
        while tokens.peek_not_eq(TokenData::EOF) && self.error_handler.can_continue() {
            self.declaration(tokens);
        }
    }

    pub fn emit(&mut self, instruction: Instruction) {
        let _ = self.emit_get(instruction);
    }

    pub fn emit_get(&mut self, instruction: Instruction) -> usize {
        self.chunk.push_instruction(instruction)
    }

    pub fn patch_jump(&mut self, slot: usize, instruction: Instruction) -> bool {
        self.chunk.patch_instruction(slot, instruction)
    }

    pub fn get_instructions_count(&self) -> usize {
        self.chunk.code.len() - 1
    }

    pub fn push_constant(&mut self, constant: Constant) -> usize {
        self.chunk.push_constant(constant)
    }

    pub fn begin_scope(&mut self) {
        self.locals.begin_scope();
    }

    pub fn end_scope(&mut self) {
        self.locals.end_scope(&mut self.chunk);
    }
}
