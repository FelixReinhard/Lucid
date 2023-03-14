mod lexing;
mod vm;
mod utils;
mod compiler;
mod args;

use crate::lexing::lexer;
use crate::args::ArgParser;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    let arg_parser = ArgParser::new(&args);
    if arg_parser.check() {
        return;
    }

    // must be ok as len >= 2 
    let filename = arg_parser.filename(); 
    let tokens_res = lexer::lex_file(&filename);
    let tokens;
    if let Err(error) = tokens_res {
        println!("{:?}", error);
        return;
    } else {
        tokens = tokens_res.unwrap();
    }

    if arg_parser.tokens() { 
        println!("{:?}", tokens);
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


