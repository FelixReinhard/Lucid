pub struct ArgParser {
    bytecode: bool,
    tokens: bool,
    stack: bool,
    filename: String,
    error: bool,
    print_res: bool,
}

impl ArgParser {

    pub fn new(args: &Vec<String>) -> ArgParser {
        let (mut bytecode, mut tokens, mut stack) = (false, false, false);
        let mut error = false;
        let mut print_res = false;
        for s in args {
            match s.as_str() {
                "--bytecode" => bytecode = true,
                "--tokens" => tokens = true,
                "--stack" => stack = true,
                "--print-result" => print_res = true, 
                _ => {}
            }
        }
        let mut filename = "none".to_string();
        if args.len() < 2 || args.len() > 5 {
            error = true;
        } else {
            if args[1].ends_with(".lucid") {
                filename = args[1].clone();
            } else {
                error = true;
            }
        }


        ArgParser{bytecode, tokens, stack, filename, error, print_res}
    }

    pub fn byte_code(&self) -> bool {
        self.bytecode
    }

    pub fn tokens(&self) -> bool {
        self.tokens
    }

    pub fn stack(&self) -> bool {
        self.stack 
    }

    pub fn filename(&self) -> &String {
        &self.filename
    }

    pub fn check(&self) -> bool {
        if self.error {
            self.wrong_args();
            true 
        } else {
            false
        }
    }

    pub fn print_res(&self) -> bool {
        self.print_res
    }

    fn wrong_args(&self) {
        println!("Usage: lucid <file>.lucid [ARGS]\n");
        println!("ARGS : --tokens");
        println!("     : --bytecode");
        println!("     : --stack");
    }
}
