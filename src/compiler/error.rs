use crate::compiler::tokenstream::TokenStream;
use crate::lexing::lexer::TokenData;
use crate::utils::LangError;

pub struct ErrorHandler {
    had_error: bool,
    panic_mode: bool,
    error: LangError,
}

impl ErrorHandler {
    pub fn new() -> ErrorHandler {
        ErrorHandler {
            had_error: false,
            panic_mode: false,
            error: LangError::None,
        }
    }

    pub fn ok(&mut self) -> bool {
        !self.had_error 
    }

    // Can the compiler continue after a declaration.
    // Maybe it encountered an error that was not possible to syncronize from.
    pub fn can_continue(&self) -> bool {
        !self.had_error 
    }

    pub fn syncronize(&self, tokens: &mut TokenStream) {
        while tokens.dont_match_tokens(vec![
            TokenData::EOF,
            TokenData::Semicol,
            TokenData::Keyword("let"),
        ]) {}
    }

    pub fn report_error(&mut self, error: LangError, tokens: &mut TokenStream) {
        self.error = error;
        self.had_error = true;
        self.error.print();
        self.syncronize(tokens);
    }
}
