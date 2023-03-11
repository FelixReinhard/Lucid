use crate::compiler::core::Compiler;
use crate::compiler::tokenstream::TokenStream;
use crate::lexing::lexer::{Token, TokenData};
use crate::utils::LangError;
use crate::vm::instructions::Instruction;

impl Compiler {
    pub fn declaration(&mut self, tokens: &mut TokenStream) {
        self.statement(tokens);
    }

    fn statement(&mut self, tokens: &mut TokenStream) {
        let peeked;
        if let Some(t) = tokens.peek() {
            peeked = t;
        } else {
            self.error_handler.report_error(
                LangError::UnknownParsing("Tried parsing statement, but couldnt get next Token."),
                tokens,
            );
            return;
        }
        match peeked.tk {
            TokenData::DEBUG => {
                tokens.next();
                self.expression(tokens);
                tokens.consume(TokenData::Semicol, &mut self.error_handler);
                self.emit(Instruction::DEBUG);
            }
            TokenData::Keyword("let") => self.var_declaration(tokens),
            _ => self.expression_statement(tokens),
        }
    }

    fn expression_statement(&mut self, tokens: &mut TokenStream) {
        self.expression(tokens);
        tokens.consume(TokenData::Semicol, &mut self.error_handler);
    }

    fn var_declaration(&mut self, tokens: &mut TokenStream) {
        // already checked that next token is "let"
        tokens.next();
        let var_name = tokens.consume_identifier(&mut self.error_handler);
        
        if tokens.match_token(TokenData::Equals) {
            self.expression(tokens);
        } else {
            // emit a nil
            self.emit(Instruction::Constant(2))
        }
        tokens.consume(TokenData::Semicol, &mut self.error_handler);
        let var_pointer = self.globals.put(var_name);
        self.emit(Instruction::DefGlobal(var_pointer));
    }
}
