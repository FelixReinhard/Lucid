use crate::utils::LangError;
use crate::utils::Value;

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Instruction {
    DEBUG, 
    Dummy, // used for patching jumps
    Return,
    Constant(usize),
    Negate,
    LogicAnd,
    LogicOr,
    BitOr,
    BitAnd,
    ShiftLeft,
    ShiftRight,
    Equal,
    Less,
    Greater,
    Not,
    Add,
    Mult,
    Div,
    Sub,
    Mod,
    Pow,
    Pop,
    DefGlobal(usize), // usize points to vms global table
    GetGlobal(usize),
    SetGlobal(usize),
    SetLocal(usize),
    GetLocal(usize),
    JumpIfFalse(usize),
    Jump(usize),
    JumpTo(usize), // sets ip
}

impl Instruction {
    pub fn unary_op(&self, operand: Value) -> Result<Value, LangError> {
        match self {
            Instruction::Not => match operand {
                Value::Bool(v) => Ok(Value::Bool(!v)),
                _ => Err(LangError::RuntimeMessage("Cannot negate(!) non bool value")),
            },
            Instruction::Negate => match operand {
                Value::Float(v) => Ok(Value::Float(-v)),
                Value::Integer(v) => Ok(Value::Integer(-v)),
                _ => Err(LangError::Runtime),
            },
            _ => {
                println!("Never happens");
                Err(LangError::Runtime)
            }
        }
    }

    pub fn binary_op(&self, left: Value, right: Value) -> Result<Value, LangError> {
        match self {
            Instruction::BitAnd => match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l & r)),
                (Value::Float(l), Value::Float(r)) => {
                    Ok(Value::Float(f64::from_bits(l.to_bits() & r.to_bits())))
                }
                _ => Err(LangError::RuntimeMessage("Cannot Bit and this type")),
            },
            Instruction::BitOr => match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l | r)),
                (Value::Float(l), Value::Float(r)) => {
                    Ok(Value::Float(f64::from_bits(l.to_bits() | r.to_bits())))
                }
                _ => Err(LangError::RuntimeMessage("Cannot Bit or this type")),
            },
            Instruction::ShiftLeft => match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l << r)),
                (Value::Float(l), Value::Float(r)) => {
                    Ok(Value::Float(f64::from_bits(l.to_bits() << r.to_bits())))
                }
                _ => Err(LangError::RuntimeMessage(
                    "Cannot shift left not integer or float type",
                )),
            },
            Instruction::ShiftRight => match (left, right) {
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l >> r)),
                (Value::Float(l), Value::Float(r)) => {
                    Ok(Value::Float(f64::from_bits(l.to_bits() >> r.to_bits())))
                }
                _ => Err(LangError::RuntimeMessage(
                    "Cannot shift right not integer or float type",
                )),
            },
            Instruction::Equal => match (left, right) {
                (Value::Bool(l), Value::Bool(r)) => Ok(Value::Bool(l == r)),
                (Value::Null, Value::Null) => Ok(Value::Bool(true)),
                (Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l == r)),
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Bool(l == r)),
                (Value::Integer(l), Value::Float(r)) => {
                    Ok(Value::Bool(r.fract() == 0.0 && l == (r as i64)))
                }
                (Value::Float(l), Value::Integer(r)) => {
                    Ok(Value::Bool(l.fract() == 0.0 && r == (l as i64)))
                }
                _ => Err(LangError::RuntimeMessage("Types cannot be equal")),
            },
            Instruction::Less => match (left, right) {
                (Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l < r)),
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Bool(l < r)),
                (Value::Integer(l), Value::Float(r)) => {
                    Ok(Value::Bool(r.fract() == 0.0 && l < (r as i64)))
                }
                (Value::Float(l), Value::Integer(r)) => {
                    Ok(Value::Bool(l.fract() == 0.0 && (l as i64) < r))
                }
                _ => Err(LangError::RuntimeMessage("Types cannot be less")),
            },
            Instruction::Greater => match (left, right) {
                (Value::Float(l), Value::Float(r)) => Ok(Value::Bool(l > r)),
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Bool(l > r)),
                (Value::Integer(l), Value::Float(r)) => {
                    Ok(Value::Bool(r.fract() == 0.0 && l > (r as i64)))
                }
                (Value::Float(l), Value::Integer(r)) => {
                    Ok(Value::Bool(l.fract() == 0.0 && (l as i64) > r))
                }
                _ => Err(LangError::RuntimeMessage("Types cannot be less")),
            },
            Instruction::Add => match (left, right) {
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l + r)),
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l + r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 + r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l + r as f64)),
                (Value::Str(s1), Value::Str(s2)) => Ok(Value::Str(Rc::new(format!("{}{}", *s1, *s2)))), 
                _ => Err(LangError::Runtime),
            },
            Instruction::Sub => match (left, right) {
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l - r)),
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l - r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 - r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l - r as f64)),
                _ => Err(LangError::Runtime),
            },
            Instruction::Mult => match (left, right) {
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l * r)),
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l * r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 * r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * r as f64)),
                _ => Err(LangError::Runtime),
            },
            Instruction::Div => match (left, right) {
                (Value::Float(l), Value::Float(r)) => {
                    if r == 0.0 {
                        Err(LangError::RuntimeDivByZero)
                    } else {
                        Ok(Value::Float(l / r))
                    }
                }
                (Value::Integer(l), Value::Integer(r)) => {
                    if l % r == 0 && r != 0 {
                        Ok(Value::Integer(l / r))
                    } else if l % r != 0 {
                        Ok(Value::Float(l as f64 / r as f64))
                    } else {
                        Err(LangError::RuntimeDivByZero)
                    }
                }
                (Value::Integer(l), Value::Float(r)) => {
                    if r == 0.0 {
                        Err(LangError::RuntimeDivByZero)
                    } else {
                        Ok(Value::Float(l as f64 / r))
                    }
                }
                (Value::Float(l), Value::Integer(r)) => {
                    if r == 0 {
                        Err(LangError::RuntimeDivByZero)
                    } else {
                        Ok(Value::Float(l / r as f64))
                    }
                }
                _ => Err(LangError::Runtime),
            },
            Instruction::Pow => match (left, right) {
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l.powf(r))),
                (Value::Integer(l), Value::Integer(r)) => {
                    if r >= 0 && r <= u32::MAX as i64 && r > 0 {
                        Ok(Value::Integer(l.pow(r as u32)))
                    } else {
                        let left_f = l as f64;
                        let right_f = r as f64;
                        Ok(Value::Float(left_f.powf(right_f)))
                    }
                }
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float((l as f64).powf(r))),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l * r as f64)),
                _ => Err(LangError::Runtime),
            },
            Instruction::Mod => match (left, right) {
                (Value::Float(l), Value::Float(r)) => Ok(Value::Float(l % r)),
                (Value::Integer(l), Value::Integer(r)) => Ok(Value::Integer(l % r)),
                (Value::Integer(l), Value::Float(r)) => Ok(Value::Float(l as f64 % r)),
                (Value::Float(l), Value::Integer(r)) => Ok(Value::Float(l % r as f64)),
                _ => Err(LangError::Runtime),
            },
            _ => {
                println!("Never happens, Illeagel binary op");
                Err(LangError::Runtime)
            }
        }
    }
}
