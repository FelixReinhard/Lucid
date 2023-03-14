// All native functions
use crate::utils::Value;

pub fn execute_native_function(id: usize, args: Vec<Value>) -> Option<Value> {
    match id {
        0 => native_println(args),
        1 => native_input(args),
        _ => None,
    }
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
    } else if vals.len() == 1 {
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
