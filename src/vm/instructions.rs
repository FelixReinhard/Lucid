use crate::vm::values::{Value};
use crate::utils::LangError;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    DEBUG, // Never compiled, only for unit tests.
    Return,
    Constant(usize),
    Negate,
    Add,
    Mult,
    Div,
    Sub,
    Mod,
    Pow,
}

impl Instruction {
    pub fn unary_op(&self, operand: Value) -> Result<Value, LangError> {
        match self {
            Instruction::Negate => {
                match operand {
                    Value::Float(v) => Ok(Value::Float(-v)),
                    Value::Integer(v) => Ok(Value::Integer(-v)),
                }
            },
            _ => {
                println!("Never happens");
                Err(LangError::Runtime)
            }
        }
    }

    pub fn binary_op(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match self {
            Instruction::Add => {
                match (left, right) {
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                    (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 + r)),
                    (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + r as f64)),
                }
            },
            Instruction::Sub => {
                match (left, right) {
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
                    (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 - r)),
                    (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l - r as f64)),
                }
            },
            Instruction::Mult => {
                match (left, right) {
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
                    (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 * r)),
                    (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * r as f64)),
                }
            },
            Instruction::Div => {
                match (left, right) {
                    (Value::Float(l), Value::Float(r)) => {
                        if r == 0.0 {
                            Err(LangError::RuntimeDivByZero)
                        } else {
                            Ok(Value::Float(l / r))
                        }
                    }, 
                    _ => Err(LangError::Runtime),
                }
            },
            Instruction::Pow => {
                match (left, right) {
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l.powf(r))),
                    (Value::Integer(l), Value::Integer(r)) => {
                        if r >= 0 && r <= u32::MAX as i64 && r > 0 {
                            Ok(Value::Integer(l.pow(r as u32)))
                        } else {
                            let left_f = l as f64;
                            let right_f = r as f64;
                            Ok(Value::Float(left_f.powf(right_f)))
                        }
                    },                   
                    (Value::Integer(l), Value::Float(r)) => Ok(Value::Float((l as f64).powf(r))),
                    (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * r as f64)),
                }
            },
            Instruction::Mod => {
                match (left, right) {
                    (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l % r)),
                    (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l % r)),
                    (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 % r)),
                    (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l % r as f64)),
                }
            },
            _ => { println!("Never happens, Illeagel binary op"); Err(LangError::Runtime) }
        }
    }
}
