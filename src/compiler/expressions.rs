use crate::compiler::core::Compiler;
use crate::compiler::tokenstream::TokenStream;
use crate::lexing::lexer::TokenData;
use crate::utils::{Constant, LangError, Value};
use crate::vm::instructions::Instruction;

use std::cell::RefCell;
use std::rc::Rc;

impl Compiler {
    pub fn expression(&mut self, tokens: &mut TokenStream) {
        self.parse_precedence(tokens, Precedence::Assign);
    }

    fn parse_precedence(&mut self, tokens: &mut TokenStream, precedence: Precedence) {
        let token;
        if let Some(t) = tokens.peek() {
            token = t;
        } else {
            self.error_handler.report_error(
                LangError::UnknownParsing("parse_precedence: Could not peek next token"),
                tokens,
            );
            return;
        }
        match token.tk {
            TokenData::ParenOpen => self.grouping(tokens),
            TokenData::Keyword("fn") => self.lambda(tokens),
            TokenData::BrackOpen => self.list(tokens),
            TokenData::Minus | TokenData::Not => self.unary(tokens),
            TokenData::I64Literal(_)
            | TokenData::F64Literal(_)
            | TokenData::BoolLiteral(_)
            | TokenData::StringLiteral(_)
            | TokenData::Keyword("null") => self.literal(tokens),
            TokenData::Identifier(_) => self.variable(tokens, precedence <= Precedence::Assign),
            TokenData::Keyword("self") => self.struct_self(tokens),
            TokenData::Keyword("new") => self.struct_instance(tokens),
            _ => {
                self.error_handler.report_error(
                    LangError::ParsingError(
                        token.line,
                        "parse_precedence: First token of expression is wrong",
                    ),
                    tokens,
                );
                return;
            }
        }

        if !self.error_handler.can_continue() {
            return;
        }
        while precedence <= tokens.get_precedence_of_peek() {
            match tokens.peek_or_none() {
                TokenData::LogicalAnd => self.logical_and(tokens),
                TokenData::LogicalOr => self.logical_or(tokens),
                TokenData::ParenOpen => self.call(tokens),
                TokenData::BrackOpen => self.list_access(tokens, precedence <= Precedence::Assign),
                TokenData::Dot => self.struct_access(tokens, precedence <= Precedence::Assign),
                _ => self.binary(tokens),
            }
        }
    }

    fn struct_self(&mut self, tokens: &mut TokenStream) {
        let selff = tokens.next().unwrap();
        self.emit(Instruction::GetSelf);
    }

    fn struct_access(&mut self, tokens: &mut TokenStream, can_assign: bool) {
        let dot = tokens.next().unwrap();
        let field = tokens.consume_identifier(&mut self.error_handler);

        let set = Instruction::StructSet(Box::new(field.clone()));
        let get = Instruction::StructGet(Box::new(field));

        let mut is_assigning = false;
        if tokens.match_token(TokenData::Equals) {
            is_assigning = true;
            self.expression(tokens);
            self.emit(set);
        } else if tokens.match_token(TokenData::MinusEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Sub);
            self.emit(set);
        } else if tokens.match_token(TokenData::PlusEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Add);
            self.emit(set);
        } else if tokens.match_token(TokenData::StarEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Mult);
            self.emit(set);
        } else if tokens.match_token(TokenData::SlashEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Div);
            self.emit(set);
        } else if tokens.match_token(TokenData::PlusPlus) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            let c = self.push_constant(Constant::Integer(1));
            self.emit(Instruction::Constant(c));
            self.emit(Instruction::Add);
            self.emit(set);
        } else if tokens.match_token(TokenData::MinusMinus) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            let c = self.push_constant(Constant::Integer(1));
            self.emit(Instruction::Constant(c));
            self.emit(Instruction::Sub);
            self.emit(set);
        } else {
            self.emit(get);
        }

        if is_assigning && !can_assign {
            self.error_handler.report_error(
                LangError::ParsingError(dot.line, "list: cannot assign here!"),
                tokens,
            );
        }
    }

    fn struct_instance(&mut self, tokens: &mut TokenStream) {
        let _ = tokens.next().unwrap();

        let struct_name = tokens.consume_identifier(&mut self.error_handler);
        let s = self.structs.get(&struct_name).unwrap();

        if tokens.match_token(TokenData::ParenOpen) {
            while !tokens.check(TokenData::ParenClose) {
                self.expression(tokens);

                if !tokens.match_token(TokenData::Coma) {
                    break;
                }
            }
            tokens.consume(TokenData::ParenClose, &mut self.error_handler);
        } else {
            for _ in 0..s.field_names.len() {
                self.emit(Instruction::Constant(2));
            }
        }

        for function in s.methods.iter() {
            if !function.is_static {
                self.emit(Instruction::FuncRef(
                    function.adress,
                    function.args_count,
                    Box::new(Rc::new(RefCell::new(function.upvalues.clone()))),
                ));
            }
        }
        let map = s.get_name_map();
        let struct_instruction = Instruction::Struct(Box::new(map));
        self.emit(struct_instruction);
    }

    fn lambda(&mut self, tokens: &mut TokenStream) {
        let fn_ = tokens.next().unwrap();

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

        let lambda = self
            .functions
            .put_lambda(jump_over_function_code + 1, arg_amount);
        if tokens.check(TokenData::Arrow) {
            self.arrow_block_fn(tokens);
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
        for _ in 0..arg_amount + 1 {
            self.emit(Instruction::Pop);
        }

        self.emit(Instruction::Constant(2));
        self.emit(Instruction::JumpRe);
        self.patch_jump(
            jump_over_function_code,
            Instruction::JumpTo(self.get_instructions_count() + 1),
        );
        if let Some(function) = self.functions.get_lambda(lambda) {
            self.emit(Instruction::FuncRef(
                function.adress,
                function.args_count,
                Box::new(Rc::new(RefCell::new(function.upvalues.clone()))),
            ));
        }

        self.functions.exit_function();
        self.locals.end_function();
    }

    fn list_access(&mut self, tokens: &mut TokenStream, can_assign: bool) {
        let brack = tokens.next().unwrap();

        self.expression(tokens);

        tokens.consume(TokenData::BrackClose, &mut self.error_handler);

        let set = Instruction::SetList;
        let get = Instruction::AccessList;

        let mut is_assigning = false;
        if tokens.match_token(TokenData::Equals) {
            is_assigning = true;
            self.expression(tokens);
            self.emit(Instruction::SetList);
        } else if tokens.match_token(TokenData::MinusEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Sub);
            self.emit(set);
        } else if tokens.match_token(TokenData::PlusEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Add);
            self.emit(set);
        } else if tokens.match_token(TokenData::StarEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Mult);
            self.emit(set);
        } else if tokens.match_token(TokenData::SlashEquals) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Div);
            self.emit(set);
        } else if tokens.match_token(TokenData::PlusPlus) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            let c = self.push_constant(Constant::Integer(1));
            self.emit(Instruction::Constant(c));
            self.emit(Instruction::Add);
            self.emit(set);
        } else if tokens.match_token(TokenData::MinusMinus) {
            is_assigning = true;
            self.emit(Instruction::Dup(2));
            self.emit(get);
            let c = self.push_constant(Constant::Integer(1));
            self.emit(Instruction::Constant(c));
            self.emit(Instruction::Sub);
            self.emit(set);
        } else {
            self.emit(get);
        }
        if is_assigning && !can_assign {
            self.error_handler.report_error(
                LangError::ParsingError(brack.line, "list: cannot assign here!"),
                tokens,
            );
        }
    }

    // we push each expression onto the stack then after we
    fn list(&mut self, tokens: &mut TokenStream) {
        let _ = tokens.next().unwrap();
        let mut len = 0;
        while !tokens.check(TokenData::BrackClose) {
            self.expression(tokens);
            len += 1;

            if !tokens.match_token(TokenData::Coma) {
                break;
            }
        }
        tokens.consume(TokenData::BrackClose, &mut self.error_handler);
        self.emit(Instruction::DefList(len));
    }

    fn call(&mut self, tokens: &mut TokenStream) {
        let _ = tokens.next().unwrap();

        let args_count = self.call_arguments(tokens);
        tokens.consume(TokenData::ParenClose, &mut self.error_handler);
        self.emit(Instruction::CallFunc(args_count));
    }

    fn call_arguments(&mut self, tokens: &mut TokenStream) -> u32 {
        let mut args = 0;

        while !tokens.check(TokenData::ParenClose) {
            args += 1;
            self.expression(tokens);

            if !tokens.match_token(TokenData::Coma) {
                break;
            }
        }
        args
    }

    fn logical_and(&mut self, tokens: &mut TokenStream) {
        let _ = tokens.next();

        let jump = self.emit_get(Instruction::Dummy);

        self.emit(Instruction::Pop);
        self.parse_precedence(tokens, Precedence::LogicAnd);

        self.patch_jump(
            jump,
            Instruction::JumpIfFalse(self.get_instructions_count() - jump),
        );
    }
    // By De Morgans law
    fn logical_or(&mut self, tokens: &mut TokenStream) {
        let _ = tokens.next();

        self.emit(Instruction::Not);
        let jump = self.emit_get(Instruction::Dummy);

        self.emit(Instruction::Pop);
        self.parse_precedence(tokens, Precedence::LogicOr);

        self.emit(Instruction::Not);
        self.patch_jump(
            jump,
            Instruction::JumpIfFalse(self.get_instructions_count() - jump),
        );

        self.emit(Instruction::Not);
    }

    fn binary(&mut self, tokens: &mut TokenStream) {
        let operator = tokens.next().unwrap();

        self.parse_precedence(tokens, Compiler::get_precedence(&operator.tk).higher());
        match operator.tk {
            TokenData::Minus => self.emit(Instruction::Sub),
            TokenData::Plus => self.emit(Instruction::Add),
            TokenData::Times => self.emit(Instruction::Mult),
            TokenData::Slash => self.emit(Instruction::Div),
            TokenData::Power => self.emit(Instruction::Pow),
            TokenData::Percent => self.emit(Instruction::Mod),
            TokenData::LogicalOr => self.emit(Instruction::LogicOr),
            TokenData::LogicalAnd => self.emit(Instruction::LogicAnd),
            TokenData::Less => self.emit(Instruction::Less),
            TokenData::Greater => self.emit(Instruction::Greater),
            TokenData::Eq => self.emit(Instruction::Equal),
            TokenData::Geq => {
                self.emit(Instruction::Less);
                self.emit(Instruction::Not);
            }
            TokenData::Leq => {
                self.emit(Instruction::Greater);
                self.emit(Instruction::Not);
            }
            TokenData::Or => self.emit(Instruction::BitOr),
            TokenData::And => self.emit(Instruction::BitAnd),
            TokenData::ShiftRight => self.emit(Instruction::ShiftRight),
            TokenData::ShiftLeft => self.emit(Instruction::ShiftLeft),
            _ => self.error_handler.report_error(
                LangError::ParsingError(operator.line, "Invalid binary op"),
                tokens,
            ),
        }
    }

    fn variable_operations(
        &mut self,
        tokens: &mut TokenStream,
        can_assign: bool,
        get: Instruction,
        set: Instruction,
        line: u32,
    ) {
        let mut is_assigning = false;
        if tokens.match_token(TokenData::Equals) {
            is_assigning = true;
            self.expression(tokens);
            self.emit(set);
        } else if tokens.match_token(TokenData::MinusEquals) {
            is_assigning = true;
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Sub);
            self.emit(set);
        } else if tokens.match_token(TokenData::PlusEquals) {
            is_assigning = true;
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Add);
            self.emit(set);
        } else if tokens.match_token(TokenData::StarEquals) {
            is_assigning = true;
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Mult);
            self.emit(set);
        } else if tokens.match_token(TokenData::SlashEquals) {
            is_assigning = true;
            self.emit(get);
            self.expression(tokens);
            self.emit(Instruction::Div);
            self.emit(set);
        } else if tokens.match_token(TokenData::PlusPlus) {
            is_assigning = true;
            self.emit(get);
            let c = self.push_constant(Constant::Integer(1));
            self.emit(Instruction::Constant(c));
            self.emit(Instruction::Add);
            self.emit(set);
        } else if tokens.match_token(TokenData::MinusMinus) {
            is_assigning = true;
            self.emit(get);
            let c = self.push_constant(Constant::Integer(1));
            self.emit(Instruction::Constant(c));
            self.emit(Instruction::Sub);
            self.emit(set);
        } else {
            self.emit(get);
        }
        if is_assigning && !can_assign {
            self.error_handler.report_error(
                LangError::ParsingError(line, "variable: cannot assign here!"),
                tokens,
            );
        }
    }

    fn variable(&mut self, tokens: &mut TokenStream, can_assign: bool) {
        // ok to unwrap as check has been done
        let identifier = tokens.next().unwrap();

        if let TokenData::Identifier(ident) = identifier.tk {
            // first check if a global is found
            if !self.locals.is_global_scope() {
                if let Some(slot) = self.locals.get_local(&ident) {
                    self.variable_operations(
                        tokens,
                        can_assign,
                        Instruction::GetLocal(slot),
                        Instruction::SetLocal(slot),
                        identifier.line,
                    );
                    // self.local(tokens, can_assign, slot, identifier.line);
                    return;
                }
            }

            if let Some(slot) = self.globals.get(&ident) {
                self.variable_operations(
                    tokens,
                    can_assign,
                    Instruction::GetGlobal(slot),
                    Instruction::SetGlobal(slot),
                    identifier.line,
                );
                // self.global(tokens, can_assign, slot, identifier.line);
            } else if let Some(function) = self.functions.get(&ident) {
                if function.is_native {
                    self.emit(Instruction::NativeRef(function.id, function.args_count));
                } else {
                    // a function also takes a Heap allocated list of upvalues stored in the
                    // function itself.
                    // So As we have a list of upvalues we copy them into this funcref when the
                    // instruction is executed it will copy the corresponding variables from the
                    // stack into the Value::Func that gets put onto the stack
                    //
                    // let upvalues = function.upvalues.clone();
                    self.emit(Instruction::FuncRef(
                        function.adress,
                        function.args_count,
                        Box::new(Rc::new(RefCell::new(function.upvalues.clone()))),
                    ));
                }
                // First check if this variable is maybe in the locals in a higher call frame
                // if yes then it returns the place on the stack of the variable in relation
                // to the callframe and the amount of call frames one needs to go up
            } else if let Some((index, call_frame_diff)) = self.locals.get_upvalue(&ident) {
                // Here we have the index in relation to the callframe that is located
                // call_frame_diff above the current callframe.

                // now if call_frame_diff is 1 then that means we capture a variable that is a
                // local in the current call_frame for example
                //
                // fn test() {
                //  let x = 10;
                //  fn test2() => x;
                // }
                //
                let slot = self.functions.add_up_value(index, call_frame_diff);

                self.variable_operations(
                    tokens,
                    can_assign,
                    Instruction::GetUpvalue(slot),
                    Instruction::SetUpvalue(slot),
                    identifier.line,
                );
            } else if let Some(s) = self.structs.get(&ident) {
                // Static method, 
                tokens.consume(TokenData::Dot, &mut self.error_handler);
                if !self.error_handler.ok() {
                    return;
                }

                let function_name = tokens.consume_identifier(&mut self.error_handler);
                if !s.has_static_method(&function_name) {
                    self.error_handler.report_error(
                        LangError::ParsingError(identifier.line, "struct does not have this method"),
                        tokens,
                    );
                    return;
                }

                if let Some(function) = self.functions.get(&function_name) {
                    self.emit(Instruction::FuncRef(
                        function.adress,
                        function.args_count,
                        Box::new(Rc::new(RefCell::new(function.upvalues.clone()))),
                    ));
                } else {
                    self.error_handler.report_error(
                        LangError::ParsingError(identifier.line, "Method wasnt found"),
                        tokens,
                    );
                    return;
                }
            } 

            else {
                self.error_handler.report_error(
                    LangError::ParsingError(identifier.line, "variable: Undefined variable!."),
                    tokens,
                );
            }
        }
    }

    fn unary(&mut self, tokens: &mut TokenStream) {
        // ok to unwrap existance has already been checked
        let operator = tokens.next().unwrap();
        self.parse_precedence(tokens, Precedence::Unary);

        match operator.tk {
            TokenData::Minus => self.emit(Instruction::Negate),
            TokenData::Not => self.emit(Instruction::Not),
            _ => {} // unreachable
        }
    }

    fn literal(&mut self, tokens: &mut TokenStream) {
        // unwrap is safe here as a check has been performed prior to this call
        let token = tokens.next().unwrap();
        let constant_pointer = match token.tk {
            TokenData::BoolLiteral(val) => {
                if val {
                    0
                } else {
                    1
                }
            }
            TokenData::F64Literal(val) => self.push_constant(Constant::Float(val)),
            TokenData::I64Literal(val) => self.push_constant(Constant::Integer(val)),
            TokenData::Keyword("null") => self.push_constant(Constant::Null),
            TokenData::StringLiteral(s) => self.push_constant(Constant::Str(Rc::new(s))),
            _ => 0,
        };
        self.emit(Instruction::Constant(constant_pointer));
    }

    fn grouping(&mut self, tokens: &mut TokenStream) {
        tokens.next();
        self.expression(tokens);
        tokens.consume(TokenData::ParenClose, &mut self.error_handler);
    }

    fn get_precedence(token: &TokenData) -> Precedence {
        match token {
            TokenData::Minus | TokenData::Plus => Precedence::Term,
            TokenData::Slash | TokenData::Times | TokenData::Percent => Precedence::Factor,
            TokenData::Power => Precedence::Power,
            TokenData::LogicalAnd => Precedence::LogicAnd,
            TokenData::LogicalOr => Precedence::LogicOr,
            TokenData::Eq | TokenData::Neq => Precedence::Equality,
            TokenData::Geq | TokenData::Greater | TokenData::Leq | TokenData::Less => {
                Precedence::Compare
            }
            TokenData::Or => Precedence::BitOr,
            TokenData::And => Precedence::BitAnd,
            TokenData::ShiftLeft | TokenData::ShiftRight => Precedence::Shift,
            TokenData::Keyword("fn") => Precedence::Lambda,
            TokenData::Keyword("new") => Precedence::Call,
            _ => Precedence::None,
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None,
    Assign,   // =
    Lambda,   // fn
    LogicOr,  // ||
    LogicAnd, // &&
    BitOr,    // |
    BitAnd,   // &
    Equality, // == !=
    Compare,  // >= > <= <
    Shift,    // << >>
    Term,     // + -
    Factor,   // * / %
    Power,    // **
    Cast,     // as
    Unary,    // ! -
    New,      // new Keyword
    Call,     // () .
    Primary,
    Error,
}

impl Precedence {
    fn higher(&self) -> Precedence {
        match self {
            Precedence::Assign => Precedence::Lambda,
            Precedence::Lambda => Precedence::LogicOr,
            Precedence::LogicOr => Precedence::LogicAnd,
            Precedence::LogicAnd => Precedence::BitOr,
            Precedence::BitOr => Precedence::BitAnd,
            Precedence::BitAnd => Precedence::Equality,
            Precedence::Equality => Precedence::Compare,
            Precedence::Compare => Precedence::Shift,
            Precedence::Shift => Precedence::Term,
            Precedence::Term => Precedence::Factor,
            Precedence::Factor => Precedence::Power,
            Precedence::Power => Precedence::Cast,
            Precedence::Cast => Precedence::Unary,
            Precedence::Unary => Precedence::New,
            Precedence::New => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
            _ => Precedence::Error,
        }
    }
}
