use crate::lexer::{Token, TokenData};
use crate::utils::LangError;
use crate::vm::chunk::Chunk;
use crate::vm::instructions::Instruction;
use crate::vm::values::*;

pub struct Compiler {
    tokens: Vec<Token>,
    compiling_chunk: Chunk,
    current_token: usize,
    had_error: bool,
    error_type: LangError,
}

enum Precendence {
    Assign,
    Lambda,
    LogicOr,
    LogicAnd,
    BitOr,
    BitAnd,
    Equality,
    Compare,
    Shift,
    Term,
    Factor,
    Power,
    Cast,
    Unary,
    Call,
    Primary,
}

impl Compiler {
    pub fn new(tks: Vec<Token>) -> Compiler {
        Compiler {
            tokens: tks,
            compiling_chunk: Chunk::new(),
            current_token: 0,
            had_error: false,
            error_type: LangError::Unknown,
        }
    }

    pub fn compile(mut self) -> Result<Chunk, LangError> {
        let chunk = self.current_chunk();
        Ok(self.compiling_chunk)
    }

    fn expression(&mut self) {}

    fn number(&mut self) {
        // Ok to unwrap as this was checked before calling number.
        let token = self.advance().unwrap();
        let line = token.line;
        let c;
        match token.tk {
            TokenData::F64Literal(value) => c = self.add_constant(Constant::Float(value)),
            TokenData::I64Literal(value) => c = self.add_constant(Constant::Integer(value)),
            _ => {
                c = 0;
                self.error(line, LangError::ParsingError("Wrong token in number parsing"))
            }
        }
        self.emit(Instruction::Constant(c));
    }

    fn grouping(&mut self) {
        let token = self.advance().unwrap();
        let line = token.line;
        self.expression();
        if let Err(err) = self.consume(TokenData::ParenClose) {
            self.error(line, err); 
        }
    }

    fn unary(&mut self) {
        let (operator, line) = self.advance_and_line().unwrap(); 
        self.expression();

        match operator {
            TokenData::Minus => self.emit(Instruction::Negate),
            _ => self.error(line, LangError::ParsingError("Wrong unary operator")),
        }
    }

    // Helper Functions =======================================
    //
    //

    fn error(&mut self, line: u32, error: LangError) {
        self.had_error = true;
        self.error_type = error;
    }

    fn consume(&mut self, tk: TokenData) -> Result<(), LangError> {
        if let Some(peeked) = self.peek(0) {
            if peeked.tk.is_eq(&tk) {
                let _ = self.advance();
                return Ok(());
            }
        }
        Err(LangError::ParsingConsume(tk))
    }

    fn advance(&mut self) -> Option<&Token> {
        if self.current_token >= self.tokens.len() {
            return None;
        }
        self.current_token += 1;
        Some(&self.tokens[self.current_token - 1])
    }

    fn advance_and_line(&mut self) -> Option<(TokenData, u32)> {
        let token = self.advance()?;
        Some((token.tk.clone(), token.line))
    }

    fn peek(&self, i: usize) -> Option<&Token> {
        if self.current_token + i >= self.tokens.len() {
            return None;
        }
        Some(&self.tokens[self.current_token + i])
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    fn emit(&mut self, instruction: Instruction) {
        let _ = self.emit_get(instruction);
    }

    fn emit_get(&mut self, instruction: Instruction) -> usize {
        self.current_chunk().push_instruction(instruction)
    }

    fn add_constant(&mut self, constant: Constant) -> usize {
        self.current_chunk().push_constant(constant)
    }
}
