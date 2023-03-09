use crate::lexing::lexer::TokenData;
#[derive(Debug)]
pub enum LangError {
    Unknown,
    LexingError,
    Runtime,
    RuntimeDivByZero,
    RuntimeArithmetic(&'static str),
    ParsingError(&'static str),
    ParsingConsume(TokenData)
}
