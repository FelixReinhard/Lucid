use crate::lexing::lexer::TokenData;
use std::cell::RefCell;
use std::collections::HashMap;
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
    ParsingConsume(u32, TokenData),
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

#[derive(Debug, Clone)]
pub enum UpValue {
    Local(usize),     // stack slot
    Recursive(usize), // index of the UpValue one call frame above
}

pub type List = Rc<Box<RefCell<Vec<Value>>>>;
// A value that is shared between more then one stack object.
pub type SVal = Box<Rc<RefCell<Value>>>;
pub type UpValueList = Box<Rc<RefCell<Vec<UpValue>>>>;

#[derive(Debug, Clone)]
pub enum Value {
    Float(f64),
    Integer(i64),
    Bool(bool),
    Str(Rc<String>),
    Func(usize, u32, List),
    NativeFunc(usize, u32),
    Null,
    List(List),
    Shared(SVal),
    StructInstance(List, Box<HashMap<String, usize>>), // Each instance has a list of its values behind a Rc
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Self::NativeFunc(id, _) => format!("native fn <{}>", id),
            Self::Float(f) => format!("{}", f),
            Self::Integer(i) => format!("{}", i),
            Self::Bool(b) => format!("{}", b),
            Self::Null => "Null".to_string(),
            Self::Str(s) => format!("{}", s),
            Self::Func(name, _, _) => format!("fn: <{}>", name),
            Self::Shared(val) => val.borrow().to_string(),
            Self::StructInstance(ls, map) => {
                let mut s = format!(
                    "struct names: ({}), values: ({})",
                    ls.borrow().iter().fold(String::new(), |acc, x| format!(
                        "{}{}",
                        acc,
                        format!(", {}", x.to_string())
                    )),
                    map.iter().fold(String::new(), |acc, (key, value)| format!(
                        "{}, {}{}",
                        acc, key, value
                    )),
                );
                s.remove(1);
                s.remove(1);
                s
            }
            Self::List(ls) => {
                let mut s = format!(
                    "[{}]",
                    ls.borrow().iter().fold(String::new(), |acc, x| format!(
                        "{}{}",
                        acc,
                        format!(", {}", x.to_string())
                    ))
                );
                s.remove(1);
                s.remove(1);
                s
            }
        }
    }

    pub fn to_debug(&self) -> String {
        match self {
            Self::NativeFunc(id, _) => format!("native fn <{}>", id),
            Self::Float(f) => format!("Float({})", f),
            Self::Integer(i) => format!("Int({})", i),
            Self::Bool(b) => format!("Bool({})", b),
            Self::Null => "Null".to_string(),
            Self::Str(s) => format!("Str({})", s),
            Self::Func(name, _, _) => format!("fn: <{}>", name),
            Self::Shared(val) => val.borrow().to_debug(),
            Self::StructInstance(ls, map) => {
                let mut s = format!(
                    "struct names: ({}), values: ({})",
                    ls.borrow().iter().fold(String::new(), |acc, x| format!(
                        "{}{}",
                        acc,
                        format!(", {}", x.to_string())
                    )),
                    map.iter().fold(String::new(), |acc, (key, value)| format!(
                        "{}, {}{}",
                        acc, key, value
                    )),
                );
                s.remove(1);
                s.remove(1);
                s
            }
            Self::List(ls) => {
                let mut s = format!(
                    "[{}]",
                    ls.borrow().iter().fold(String::new(), |acc, x| format!(
                        "{}{}",
                        acc,
                        format!(", {}", x.to_string())
                    ))
                );
                s.remove(1);
                s.remove(1);
                s
            }
        }
    }
    // if value can be interpreted as a bool return this value,
    pub fn is_falsey(&self) -> Option<bool> {
        match self {
            Self::Float(f) => Some(*f == 0.0),
            Self::Integer(i) => Some(*i == 0),
            Self::Bool(b) => Some(!*b),
            _ => None,
        }
    }
}
