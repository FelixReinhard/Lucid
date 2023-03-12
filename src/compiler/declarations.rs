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
            TokenData::CurlyOpen => self.block(tokens),
            TokenData::Arrow => self.arrow_block(tokens),
            TokenData::Keyword("if") => self.if_statement(tokens),
            TokenData::Keyword("while") => self.while_statement(tokens),
            _ => self.expression_statement(tokens),
        }
    }

    fn while_statement(&mut self, tokens: &mut TokenStream) {
        let while_ = tokens.next().unwrap();

        let loop_start = self.get_instructions_count();
        self.expression(tokens);

        let jump_exit = self.emit_get(Instruction::Dummy);
        self.emit(Instruction::Pop);

        // Block
        if tokens.check(TokenData::Arrow) {
            self.arrow_block(tokens);
        } else if tokens.check(TokenData::CurlyOpen) {
            self.block(tokens);
        } else {
            self.error_handler.report_error(
                LangError::ParsingError(
                    while_.line,
                    "Wrong token after while statement. Expected '{' or '=>'!",
                ),
                tokens,
            );
            return;
        }
        self.emit(Instruction::JumpTo(loop_start + 1));
        self.patch_jump(jump_exit, Instruction::JumpIfFalse(self.get_instructions_count() - jump_exit));
        self.emit(Instruction::Pop);
    }

    fn if_statement(&mut self, tokens: &mut TokenStream) {
        let if_ = tokens.next().unwrap();

        self.expression(tokens);

        let jump = self.emit_get(Instruction::Dummy);
        self.emit_get(Instruction::Pop);

        if tokens.check(TokenData::Arrow) {
            self.arrow_block(tokens);
        } else if tokens.check(TokenData::CurlyOpen) {
            self.block(tokens);
        } else {
            self.error_handler.report_error(
                LangError::ParsingError(
                    if_.line,
                    "Wrong token after if statement. Expected '{' or '=>'!",
                ),
                tokens,
            );
            return;
        }
        // if there is an else clause we add after the block a jump that jumps over the else block
        // we now jump over the original block AND the unconditional jump over the else block.
        if tokens.match_token(TokenData::Keyword("else")) {
            let else_jump = self.emit_get(Instruction::Dummy);
            self.patch_jump(
                jump,
                Instruction::JumpIfFalse(self.get_instructions_count() - jump),
            );
            self.emit(Instruction::Pop);

            // check the block and compile it. 
            if tokens.check(TokenData::Arrow) {
                self.arrow_block(tokens);
            } else if tokens.check(TokenData::CurlyOpen) {
                self.block(tokens);
            } else {
                self.error_handler.report_error(
                    LangError::ParsingError(
                        if_.line,
                        "Wrong token after if statement. Expected '{' or '=>'!",
                    ),
                    tokens,
                );
                return;
            }

            self.patch_jump(else_jump, Instruction::Jump(self.get_instructions_count() - else_jump));
        }
    }

    fn arrow_block(&mut self, tokens: &mut TokenStream) {
        self.locals.begin_scope();
        tokens.next();

        self.statement(tokens);

        self.locals.end_scope(&mut self.chunk);
    }

    fn block(&mut self, tokens: &mut TokenStream) {
        self.locals.begin_scope();
        tokens.next();

        while !tokens.check(TokenData::CurlyClose) && !tokens.check(TokenData::EOF) {
            self.declaration(tokens);
        }

        tokens.consume(TokenData::CurlyClose, &mut self.error_handler);
        self.locals.end_scope(&mut self.chunk);
    }

    fn expression_statement(&mut self, tokens: &mut TokenStream) {
        self.expression(tokens);
        self.emit(Instruction::Pop);
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
        // Global var
        if self.locals.is_global_scope() {
            let var_pointer = self.globals.put(var_name);
            self.emit(Instruction::DefGlobal(var_pointer));
        } else {
            self.locals.add_local(var_name);
        }
    }
}
