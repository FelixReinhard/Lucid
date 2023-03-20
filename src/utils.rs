use crate::lexing::lexer::TokenData;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::collections::VecDeque;
use crate::lexer::Token;

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
            Self::StructInstance(ls, _) => {
                let s = format!(
                    "struct : ({})",
                    ls.borrow().iter().fold(String::new(), |acc, x| format!(
                        "{}{}",
                        acc,
                        format!("{}, ", x.to_string())
                    )),
                );
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
                if s.len() > 2 {
                    s.remove(1);
                    s.remove(1);
                }
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
                let s = format!(
                    "struct names: ({}), values: ({})",
                    ls.borrow().iter().fold(String::new(), |acc, x| format!(
                        "{}{}",
                        acc,
                        format!("{}, ", x.to_string())
                    )),
                    map.iter().fold(String::new(), |acc, (key, value)| format!(
                        "{}:{}, {}",
                        key, value, acc
                    )),
                );
                s
            }
            Self::List(ls) => {
                let s = format!(
                    "[{}]",
                    ls.borrow().iter().fold(String::new(), |acc, x| format!(
                        "{}{}",
                        acc,
                        format!(", {}", x.to_string())
                    ))
                );
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

const LINUX_LIB_PATH: &str = "";
const WINDOWS_LIB_PATH: &str = "";

pub fn standard_path() -> PathBuf {
    let mut buf = PathBuf::new();
    buf.push(match std::env::consts::OS {
        "linux" => LINUX_LIB_PATH,
        "windows" => WINDOWS_LIB_PATH,
        _ => "",
    });
    buf
}

use std::path::PathBuf;

pub fn get_import_path(name: String) -> String {
    let standard_path = standard_path();    
    // replace the "
    let name = name.replace("\"", "");
    let mut path_from_import_statement = PathBuf::new();
    let mut is_std = false;
    for s in name.split("::") {
        if s == "std" {
            is_std = true;
        }
        path_from_import_statement.push(s);
    }
    if is_std {
        let mut buf = PathBuf::new();
        buf.push(standard_path);
        buf.push(path_from_import_statement);
        path_from_import_statement = buf;
    }

    format!(
        "{}.lucid",
        path_from_import_statement
            .into_os_string()
            .into_string()
            .unwrap()
    )
}

pub fn print_tokens(tokens: &VecDeque<Token>) {
    let mut current_file = String::new();
    for token in tokens {
        if current_file != token.filename {
            println!("\nFile: {}", token.filename);
            println!("=======================");
            current_file = token.filename.clone();
        }
        println!(" - {:?}", token.tk);
    }
    println!("");
}
