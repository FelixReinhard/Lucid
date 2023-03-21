// All native functions
use crate::utils::Value;
use std::boxed::Box;
use std::cell::RefCell;
use std::rc::Rc;
use std::{thread, time};
use std::time::SystemTime;
use std::fs;
use std::path::Path;

pub fn execute_native_function(id: usize, args: Vec<Value>) -> Option<Value> {
    match id {
        0 => native_println(args),
        1 => native_input(args),
        2 => native_len(args),
        3 => native_range(args),
        4 => native_sleep(args),
        5 => native_now(args),
        6 => native_read_file(args),
        7 => native_push(args),
        8 => native_string_get_at(args),
        _ => None,
    }
}

fn native_string_get_at(args: Vec<Value>) -> Option<Value> {
    if let Some(Value::Str(s)) = args.get(1) {
        if let Some(Value::Integer(i)) = args.get(0) {
            let c = s.chars().nth(*i as usize);
            if let Some(cc) = c {
                return Some(Value::Str(Rc::new(format!("{}", cc))));
            }
        }
    }
    None
}

fn native_push(args: Vec<Value>) -> Option<Value> {
    if let Some(Value::List(ls)) = args.get(1) {
        if let Some(val) = args.get(0) {
            let mut ls_borrow = ls.borrow_mut();
            ls_borrow.push(val.clone());
            return Some(Value::Null);
        }
    }
    None
}

fn native_read_file(args: Vec<Value>) -> Option<Value> {
    if let Some(Value::Str(path)) = args.get(0) {
        if let Ok(s) = fs::read_to_string(format!("{}", path)) {
            return Some(Value::Str(Rc::new(s)));
        }
    } 
    Some(Value::Null)
}

fn native_now(_args: Vec<Value>) -> Option<Value> {
    if let Ok(res) = i64::try_from(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("millis error").as_millis()) {
        Some(Value::Integer(res))
    } else {
        None
    }
}

fn native_sleep(args: Vec<Value>) -> Option<Value> {
    if let Some(Value::Integer(duration)) = args.get(0) {
        let dur_u64: u64 = if *duration >= 0 { *duration as u64 } else { 0 };
        let duration_millis = time::Duration::from_millis(dur_u64);

        thread::sleep(duration_millis);

        return Some(Value::Null);
    }
    None
}
// Rc<Box<RefCell<Vec<Value>>>>;
fn native_range(args: Vec<Value>) -> Option<Value> {
    if args.len() == 1 {
        if let Value::Integer(x) = args[0] {
            return Some(Value::List(Rc::new(Box::new(RefCell::new(
                (0..x).map(|i| Value::Integer(i)).collect(),
            )))));
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
