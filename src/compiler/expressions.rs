use crate::compiler::core::Compiler;
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
                _ => self.binary(tokens),
            }
        }
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
            _ => self.error_handler.report_error(LangError::ParsingError(operator.line, "Invalid binary op"), tokens), 
        }
    }

    fn variable(&mut self, tokens: &mut TokenStream, can_assign: bool) {
        // ok to unwrap as check has been done
        let identifier = tokens.next().unwrap();
        match identifier.tk {
            TokenData::Identifier(ident) => {
                if let Some(pointer) = self.globals.get(&ident) {
                    if tokens.match_token(TokenData::Equals) {
                        if !can_assign {
                            self.error_handler.report_error(
                                LangError::ParsingError(
                                    identifier.line,
                                    "variable: cannot assign here!",
                                ),
                                tokens,
                            );
                            return;
                        }
                        self.expression(tokens);
                        self.emit(Instruction::SetGlobal(pointer));
                    } else {
                        self.emit(Instruction::GetGlobal(pointer));
                    }
                } else {
                    self.error_handler.report_error(
                        LangError::ParsingError(
                            identifier.line,
                            "variable: unable to get variable from globals table!.",
                        ),
                        tokens,
                    );
                }
            }
            _ => {} // unreachable
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
        let constant_pointer;
        match token.tk {
            TokenData::BoolLiteral(val) => constant_pointer = if val { 0 } else { 1 },
            TokenData::F64Literal(val) => {
                constant_pointer = self.push_constant(Constant::Float(val))
            }
            TokenData::I64Literal(val) => {
                constant_pointer = self.push_constant(Constant::Integer(val))
            }
            TokenData::Keyword("null") => constant_pointer = self.push_constant(Constant::Null),
            TokenData::StringLiteral(s) => {
                constant_pointer = self.push_constant(Constant::Str(Rc::new(s)))
            }
            _ => {
                constant_pointer = 0;
            } // unreachable
        }
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
