// All native functions
use crate::utils::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::boxed::Box;

pub fn execute_native_function(id: usize, args: Vec<Value>) -> Option<Value> {
    match id {
        0 => native_println(args),
        1 => native_input(args),
        2 => native_len(args),
        3 => native_range(args),
        _ => None,
    }
}
// Rc<Box<RefCell<Vec<Value>>>>;
fn native_range(args: Vec<Value>) -> Option<Value> {
    if args.len() == 1 {
        if let Value::Integer(x) = args[0] {
            return Some(Value::List(
                Rc::new(Box::new(RefCell::new(
                    (0..x).map(|i| Value::Integer(i)).collect()
                )))
            ));
        }
    }
    None
}

fn native_len(args: Vec<Value>) -> Option<Value> {
    if args.len() == 1 {
        if let Some(Value::List(ls)) = args.get(0) {
            let borrow = ls.borrow();
            return Some(Value::Integer(borrow.len() as i64));
        }
    }
    None
}

fn native_input(args: Vec<Value>) -> Option<Value> {
    if args.len() == 1 {
        println!("{}", args[0].to_string());
    }
    let mut line = String::new();
    let _ = std::io::stdin().read_line(&mut line).unwrap();
    let line = line.replace("\r", "").replace("\n", "");
    Some(Value::Str(std::rc::Rc::new(line)))
}

fn native_println(vals: Vec<Value>) -> Option<Value> {
    if vals.len() == 0 {
        println!();
    } else if vals.is_empty() {
        println!("{}", vals.get(0).unwrap().to_string());
    } else {
        let mut res = String::new();
        for val in vals {
            res += val.to_string().as_str();
        }
        println!("{}", res);
    }
    Some(Value::Null)
}
