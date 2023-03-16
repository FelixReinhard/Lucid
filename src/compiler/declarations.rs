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
            TokenData::Keyword("let") => self.var_declaration(tokens),
            TokenData::Keyword("struct") => self.struct_declaration(tokens),
            TokenData::Keyword("fn") => self.function(tokens),
            TokenData::CurlyOpen => self.block(tokens),
            TokenData::Arrow => self.arrow_block(tokens, false),
            TokenData::Keyword("if") => self.if_statement(tokens),
            TokenData::Keyword("while") => self.while_statement(tokens),
            TokenData::Keyword("return") => self.return_statement(tokens),
            _ => self.expression_statement(tokens),
        }
    }

    fn struct_declaration(&mut self, tokens: &mut TokenStream) {

    }

    fn return_statement(&mut self, tokens: &mut TokenStream) {
        let _ = tokens.next().unwrap();
        

        if tokens.match_token(TokenData::Semicol) {
            // return null;
            self.emit(Instruction::Constant(2));
        } else {
            // emit the value of the expression
            self.expression(tokens);
            tokens.consume(TokenData::Semicol, &mut self.error_handler)
        }
        self.emit(Instruction::Return);
    }

    fn function(&mut self, tokens: &mut TokenStream) {
        let fn_ = tokens.next().unwrap();
        // get function name
        let function_name = tokens.consume_identifier(&mut self.error_handler);

        if !self.error_handler.ok() {
            return;
        }
        // Now we write the functions code, normally one needs to jump over it
        // when calling we jump here and after that jump back
        let jump_over_function_code = self.emit_get(Instruction::Dummy);

        tokens.consume(TokenData::ParenOpen, &mut self.error_handler);
        self.locals.new_function();

        let arg_amount = self.function_parameters(tokens);

        tokens.consume(TokenData::ParenClose, &mut self.error_handler);

        self.functions
            .put(function_name, jump_over_function_code + 1, arg_amount);
        if tokens.check(TokenData::Arrow) {
            self.arrow_block(tokens, false);
        } else if tokens.check(TokenData::CurlyOpen) {
            self.block(tokens);
        } else {
            self.error_handler.report_error(
                LangError::ParsingError(
                    fn_.line,
                    "Wrong token after fn declaration. Expected '{' or '=>'!",
                ),
                tokens,
            );
            return;
        }
        // Pop of all arguments and the funcref
        for _ in 0..arg_amount+1 {
            self.emit(Instruction::Pop);
        }
        self.emit(Instruction::Constant(2));
        self.emit(Instruction::JumpRe);
        self.patch_jump(
            jump_over_function_code,
            Instruction::JumpTo(self.get_instructions_count() + 1),
        );
        self.functions.exit_function();
        self.locals.end_function();
    }

    pub fn function_parameters(&mut self, tokens: &mut TokenStream) -> u32 {
        let mut arg_count = 0;

        while !tokens.check(TokenData::ParenClose) {
            arg_count += 1;

            let var_name = tokens.consume_identifier(&mut self.error_handler);
            self.locals.add_local(var_name);
            if !tokens.match_token(TokenData::Coma) {
                break;
            }
        }
        arg_count
    }

    fn while_statement(&mut self, tokens: &mut TokenStream) {
        let while_ = tokens.next().unwrap();

        let loop_start = self.get_instructions_count();
        self.expression(tokens);

        let jump_exit = self.emit_get(Instruction::Dummy);
        self.emit(Instruction::Pop);

        // Block
        if tokens.check(TokenData::Arrow) {
            self.arrow_block(tokens, false);
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
        self.patch_jump(
            jump_exit,
            Instruction::JumpIfFalse(self.get_instructions_count() - jump_exit),
        );
        self.emit(Instruction::Pop);
    }

    fn if_statement(&mut self, tokens: &mut TokenStream) {
        let if_ = tokens.next().unwrap();

        self.expression(tokens);

        let jump = self.emit_get(Instruction::Dummy);
        self.emit_get(Instruction::Pop);

        if tokens.check(TokenData::Arrow) {
            self.arrow_block(tokens, false);
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
                self.arrow_block(tokens, false);
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

            self.patch_jump(
                else_jump,
                Instruction::Jump(self.get_instructions_count() - else_jump),
            );
        }
    }

    pub fn arrow_block(&mut self, tokens: &mut TokenStream, add_semicol_after: bool) {
        self.begin_scope();
        tokens.next();

        self.statement(tokens);
        if add_semicol_after {
            tokens.tokens.push_front(Token{tk: TokenData::Semicol, line: 1});
        }
        self.end_scope();
    }

    pub fn block(&mut self, tokens: &mut TokenStream) {
        self.begin_scope();
        tokens.next();

        while !tokens.check(TokenData::CurlyClose) && !tokens.check(TokenData::EOF) &&!tokens.tokens.is_empty() {
            self.declaration(tokens);
        }

        tokens.consume(TokenData::CurlyClose, &mut self.error_handler);
        self.end_scope();
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
