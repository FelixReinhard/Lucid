use crate::lexing::lexer::{Token, TokenData};
use crate::utils::LangError;
use crate::compiler::error::ErrorHandler;
use crate::compiler::expressions::Precedence;

use std::collections::VecDeque;

pub struct TokenStream {
    pub tokens: VecDeque<Token>
}

impl TokenStream {
    pub fn new(tokens: VecDeque<Token>) -> TokenStream {
        TokenStream{tokens}
    }
    
    pub fn next(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }
    
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(0)
    }

    pub fn peek_not_eq(&self, token_type: TokenData) -> bool {
        match self.peek() {
            Some(t) => !t.tk.is_eq(&token_type),
            None => false
        }
    }

    pub fn peek_or_none(&self) -> &TokenData {
        if let Some(tk) = self.peek() {
            &tk.tk
        } else {
            &TokenData::Empty
        }
    }

    pub fn check(&self, token_type : TokenData) -> bool {
       if let Some(tk) = self.peek() {
            if tk.tk.is_eq(&token_type) {
                return true;
            }
        }
        false
    }

    pub fn get_precedence_of_peek(&mut self) -> Precedence {
        if let Some(t) = self.peek() {
            match t.tk {
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
                TokenData::ParenOpen => Precedence::Call,
                TokenData::BrackOpen => Precedence::Call,
                TokenData::Dot => Precedence::Call,
                _ => Precedence::None,
            }
        } else {
            Precedence::None
        }
    }

    pub fn consume(&mut self, token_type: TokenData, error_handler: &mut ErrorHandler) {
        if let Some(tk) = self.peek() {
            if tk.tk.is_eq(&token_type) {
                self.next();
                return;
            } else {
                error_handler.report_error(LangError::ParsingConsume(tk.line, token_type), self);
                return;
            }
        }
        error_handler.report_error(LangError::UnknownParsing("When consuming could not peek!"), self);
    }

    // Special case as when consuming an identifier we want to mostly get a copy 
    // of the name to put that into GlobalsTable
    pub fn consume_identifier(&mut self, error_handler: &mut ErrorHandler) -> String {
        if let Some(tk) = self.peek() {
            if tk.tk.is_eq(&TokenData::Identifier("_".to_string())) {
                // in this block next() must return a Identifier, so unwrap is fine.
                let token = self.next().unwrap();
                match token.tk {
                    TokenData::Identifier(s) => return s,
                    _ => {}
                }
            } else {
                error_handler.report_error(LangError::ParsingConsume(tk.line, TokenData::Identifier("Some".to_string())), self);
            }
        }
        error_handler.report_error(LangError::UnknownParsing("When consuming could not peek!"), self);
        "".to_string()
    }

    pub fn match_token(&mut self, token_type: TokenData) -> bool {
       if let Some(tk) = self.peek() {
            if tk.tk.is_eq(&token_type) {
                self.next();
                return true;
            }
        }
        false
    }
    
    // Advance if next token is not this one 
    pub fn dont_match_token(&mut self, token_type: TokenData) -> bool {
       if let Some(tk) = self.peek() {
            if !tk.tk.is_eq(&token_type) {
                self.next();
                return true;
            }
        }
        false
    }
    
    // Only advances if none of the specified tokens are the next one
    pub fn dont_match_tokens(&mut self, token_types: Vec<TokenData>) -> bool {
        if let Some(tk) = self.peek() {
            if token_types.into_iter().all(|t| !tk.tk.is_eq(&t)) {
                self.next();
                return true;
            }
        }
        false
    }
}
