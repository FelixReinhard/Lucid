use crate::lexing::lexer::TokenData;
use std::rc::Rc;

#[derive(Debug)]
pub enum LangError {
    None,
    Unknown,
    LexingError(u32),
    Runtime,
    RuntimeMessage(&'static str),
    RuntimeDivByZero,
    RuntimeArithmetic(u32, &'static str),
    ParsingError(u32, &'static str),
    UnknownParsing(&'static str),
    ParsingConsume(u32, TokenData)
}

impl LangError {
    pub fn print(&self) {
        match self {
            Self::LexingError(x) => println!("{}: LexingError", x), 
            Self::ParsingError(x, m) => println!("{}: ParsingError({})", x, m),
            Self::ParsingConsume(x, tk) => println!("{}: ParsingConsume({:?})", x, tk),
            other => println!("{:?}", other),
        }
    }
}


#[derive(Debug)]
pub enum Constant {
    Float(f64),
    Integer(i64),
    Bool(bool),
    Null,
    Str(Rc<String>),
}

impl Constant {
    pub fn to_value(&self) -> Value {
        match self {
            Self::Float(v) => Value::Float(*v),
            Self::Integer(v) => Value::Integer(*v),
            Self::Str(s) => Value::Str(Rc::clone(s)),
            Self::Bool(b) => Value::Bool(*b),
            Self::Null => Value::Null, 
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Float(f64),
    Integer(i64),
    Bool(bool),
    Str(Rc<String>),
    Null,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Self::Float(f) => format!("{}", f),
            Self::Integer(i) => format!("{}", i),
            Self::Bool(b) => format!("{}", b),
            Self::Null => "Null".to_string(),
            Self::Str(s) => format!("{}", s),
        } 
    }
}

