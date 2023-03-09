#[derive(Debug)]
pub enum Constant {
    Float(f64),
    Integer(i64),
}

impl Constant {
    pub fn to_value(&self) -> Value {
        match self {
            Self::Float(v) => Value::Float(*v),
            Self::Integer(v) => Value::Integer(*v),
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Float(f64),
    Integer(i64),
}
