mod lexing;
mod vm;
mod utils;
mod compiler;
mod args;

use crate::lexing::lexer;
use crate::lexer::Token;
use crate::args::ArgParser;
use std::env;
use std::collections::VecDeque;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = std::env::args().collect();
    
    let arg_parser = ArgParser::new(&args);
    if arg_parser.check() {
        return;
    }

    // must be ok as len >= 2 
    let filename = arg_parser.filename(); 
    let tokens_res = lexer::lex_file(filename);
    let tokens;
    if let Err(error) = tokens_res {
        println!("{:?}", error);
        return;
    } else {
        tokens = tokens_res.unwrap();
    }

    if arg_parser.tokens() { 
        print_tokens(&tokens);
    }
    let chunk_res = compiler::core::compile(tokens); 
    let chunk;

    if let Some(c) = chunk_res {
        chunk = c;
    } else {
        return;
    }

    if arg_parser.byte_code() { 
        chunk.print_constants();
        println!("==========================");
        chunk.print_code();
    }

    let interpret_res = vm::core::interpret(chunk, arg_parser.stack()); // temp always print stack.
    if let Err(error) = interpret_res {
        println!("{:?}", error);
    } else {
        if arg_parser.print_res() {
            println!("{:?}", interpret_res.unwrap());
        }
    }
}

fn print_tokens(tokens: &VecDeque<Token>) {
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
