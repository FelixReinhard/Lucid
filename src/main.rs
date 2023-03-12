mod lexing;
mod vm;
mod utils;
mod compiler;

use crate::lexing::lexer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        wrong_args();
        return;
    }
    
    // must be ok as len >= 2 
    let filename = args.get(1).unwrap();
    let tokens_res = lexer::lex_file(&filename);
    let tokens;
    if let Err(error) = tokens_res {
        println!("{:?}", error);
        return;
    } else {
        tokens = tokens_res.unwrap();
    }

    if args.len() == 3 && args[2] == "--tokens" {
        println!("{:?}", tokens);
    }
    let chunk_res = compiler::core::compile(tokens); 
    let chunk;

    if let Some(c) = chunk_res {
        chunk = c;
    } else {
        return;
    }

    if args.len() == 3 && args[2] == "--bytecode" {
        chunk.print_constants();
        println!("==========================");
        chunk.print_code();
    }

    let interpret_res = vm::core::interpret(chunk, true); // temp always print stack.
    if let Err(error) = interpret_res {
        println!("{:?}", error);
    } else {
        println!("{:?}", interpret_res.unwrap());
    }
}

fn wrong_args() {
    println!("Usage: lucid <file>.lucid [ARGS]\n");
    println!("ARGS : --tokens");
    println!("     : --bytecode");
}
