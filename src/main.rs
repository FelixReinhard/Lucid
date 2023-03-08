mod front;
use crate::front::lexer;


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
        println!("Error while Lexing {:?}", error);
        return;
    } else {
        tokens = tokens_res.unwrap();
    }

    if args.len() == 3 && args[2] == "--tokens" {
        println!("{:?}", tokens);
    }
}

fn wrong_args() {
    println!("Usage: llucid <file>.lucid [ARGS]\n");
    println!("ARGS : --tokens");
    println!("     : --bytecode");
}
