mod lexing;
mod vm;
mod utils;
mod compiler;
mod args;

use crate::lexing::lexer;
use crate::args::ArgParser;
use std::env;

fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    // TODO remove for releases
    // if let Err(e) = update_std() {
    //     println!("{:?}", e);
    //     return;
    // }
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
        crate::utils::print_tokens(&tokens);
    }
    let chunk_res = compiler::core::compile(tokens, arg_parser.tokens()); 
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

// only when compiling from source.
// fn update_std() -> std::io::Result<()> {
//     if std::path::Path::new("std").is_dir() {
//         println!("Updating std");
//         let mut path = crate::utils::standard_path();
//         path.push("std");
//         std::fs::create_dir_all(path.clone())?;
//         for entry in std::fs::read_dir("./std")? {
//             let entry = entry?;
//             if entry.file_type()?.is_dir() {
//
//             } else {
//                 // std::fs::copy(entry.path(), path.join(entry.file_name()))?;
//             }
//         }
//     }
//     Ok(())
// }
