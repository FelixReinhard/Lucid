use crate::compiler::core::Compiler;
use crate::compiler::functions::{FunctionData, UpValue};
use crate::compiler::tokenstream::TokenStream;
use crate::lexing::lexer::TokenData;
use crate::utils::{Constant, LangError, Value};
use crate::vm::instructions::Instruction;

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
            TokenData::Minus | TokenData::Not => self.unary(tokens),
            TokenData::I64Literal(_)
            | TokenData::F64Literal(_)
            | TokenData::BoolLiteral(_)
            | TokenData::StringLiteral(_)
            | TokenData::Keyword("null") => self.literal(tokens),
            TokenData::Identifier(_) => self.variable(tokens, precedence <= Precedence::Assign),
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
                _ => self.binary(tokens),
            }
        }
    }

    fn call(&mut self, tokens: &mut TokenStream) {
        let open_paren = tokens.next().unwrap();

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
            self.emit(get);
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
                    self.emit(Instruction::FuncRef(
                        function.adress,
                        function.args_count,
                        Box::new(Rc::clone(&function.upvalues)),
                    ));
                }
            } else if let Some(upval) = self.locals.get_upvalue(&ident) {
                // we encountered a variable that is captured, so now we need to tell the current
                // function we are inside of that it needs to capture this variable, to do so we
                // simply do
                if let Some(index) = self.functions.add_upvalue(UpValue {
                    index: upval,
                    is_local: false,
                }) {
                    self.variable_operations(
                        tokens,
                        can_assign,
                        Instruction::GetUpvalue(index),
                        Instruction::SetUpvalue(index),
                        identifier.line,
                    );
                } else {
                    self.error_handler.report_error(
                        LangError::ParsingError(identifier.line, "variable: upvalue problem"),
                        tokens,
                    );
                }
            } else {
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
        tokens.consume(TokenData::Semicol, &mut self.error_handler);
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
            Precedence::Unary => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
            _ => Precedence::Error,
        }
    }
}
